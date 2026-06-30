use std::sync::{Arc, Condvar, Mutex};


/*
La struct MpMcChannel<E: Send> è una implementazione di un canale su cui possono scrivere molti produttori 
e da cui possono attingere valori molti consumatori. Tale struttura offre i seguenti metodi:

new(n: usize) -> Self    //crea una istanza del canale basato su un buffer circolare di "n" elementi

send(e: E) -> Option<()>    //invia l'elemento "e" sul canale. Se il buffer circolare è pieno, attende
                            //senza consumare CPU che si crei almeno un posto libero in cui depositare il valore
                            //Ritorna:
                                // - Some(()) se è stato possibile inserire il valore nel buffer circolare
                                // - None se il canale è stato chiuso (Attenzione: la chiusura può avvenire anche
                                //    mentre si è in attesa che si liberi spazio) o se si è verificato un errore interno

recv() -> Option<E>         //legge il prossimo elemento presente sul canale. Se il buffer circolare è vuoto,
                            //attende senza consumare CPU che venga depositato almeno un valore
                            //Ritorna:
                                // - Some(e) se è stato possibile prelevare un valore dal buffer
                                // - None se il canale è stato chiuso (Attenzione: se, all'atto della chiusura sono
                                //    già presenti valori nel buffer, questi devono essere ritornati, prima di indicare
                                //    che il buffer è stato chiuso; se la chiusura avviene mentre si è in attesa di un
                                //    valore, l'attesa si sblocca e viene ritornato None) o se si è verificato un errore interno.

shutdown() -> Option<()>    //chiude il canale, impedendo ulteriori invii di valori.
                            //Ritorna:
                                // - Some(()) per indicare la corretta chiusura
                                // - None in caso di errore interno all'implementazione del metodo.

Si implementi tale struttura dati in linguaggio Rust, senza utilizzare i canali forniti dalla libreria standard né 
da altre librerie, avendo cura di garantirne la correttezza in presenza di più thread e di non generare la condizione 
di panico all'interno dei suoi metodi.

*/

// ======== Inserisci implementazione: 



// ======== TEST


fn main() {
    println!("Hello, world!");
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    // --- Test base ---

    #[test]
    fn test_send_recv_base() {
        let ch = MpMcChannel::new(4);
        assert_eq!(ch.send(1), Some(()));
        assert_eq!(ch.recv(), Some(1));
    }

    #[test]
    fn test_ordine_fifo() {
        let ch = MpMcChannel::new(4);
        ch.send(1);
        ch.send(2);
        ch.send(3);
        assert_eq!(ch.recv(), Some(1));
        assert_eq!(ch.recv(), Some(2));
        assert_eq!(ch.recv(), Some(3));
    }

    // --- Test buffer circolare ---

    #[test]
    fn test_buffer_circolare() {
        let ch = MpMcChannel::new(2);
        ch.send(1);
        ch.send(2);
        ch.recv();       // libera uno slot
        ch.send(3);      // deve riutilizzare lo slot liberato
        assert_eq!(ch.recv(), Some(2));
        assert_eq!(ch.recv(), Some(3));
    }

    // --- Test shutdown ---

    #[test]
    fn test_shutdown_send_ritorna_none() {
        let ch = MpMcChannel::new(4);
        assert_eq!(ch.shutdown(), Some(()));
        assert_eq!(ch.send(1), None);
    }

    #[test]
    fn test_shutdown_svuota_buffer_prima() {
        let ch = MpMcChannel::new(4);
        ch.send(10);
        ch.send(20);
        ch.shutdown();
        // i valori già nel buffer devono uscire prima del None
        assert_eq!(ch.recv(), Some(10));
        assert_eq!(ch.recv(), Some(20));
        assert_eq!(ch.recv(), None);
    }

    #[test]
    fn test_shutdown_doppio() {
        let ch: MpMcChannel<i32> = MpMcChannel::new(4);
        assert_eq!(ch.shutdown(), Some(()));
        // secondo shutdown: comportamento definito dall'impl, non deve andare in panico
        let _ = ch.shutdown();
    }

    // --- Test multi-thread: produttore/consumatore ---

    #[test]
    fn test_mpmc_base() {
        let ch = MpMcChannel::new(4);
        let ch2 = ch.clone();

        let producer = thread::spawn(move || {
            for i in 0..10 {
                ch2.send(i).unwrap();
            }
        });

        let mut risultati = vec![];
        for _ in 0..10 {
            risultati.push(ch.recv().unwrap());
        }

        producer.join().unwrap();
        assert_eq!(risultati, (0..10).collect::<Vec<_>>());
    }

    #[test]
    fn test_buffer_pieno_sblocca() {
        let ch = MpMcChannel::new(2);
        let ch2 = ch.clone();

        // riempie il buffer
        ch.send(1);
        ch.send(2);

        let producer = thread::spawn(move || {
            // si blocca finché non si libera uno slot
            ch2.send(3)
        });

        thread::sleep(Duration::from_millis(50));
        ch.recv(); // libera uno slot
        assert_eq!(producer.join().unwrap(), Some(()));
    }

    #[test]
    fn test_buffer_vuoto_sblocca() {
        let ch = MpMcChannel::new(4);
        let ch2 = ch.clone();

        let consumer = thread::spawn(move || {
            // si blocca finché non arriva un valore
            ch2.recv()
        });

        thread::sleep(Duration::from_millis(50));
        ch.send(42);
        assert_eq!(consumer.join().unwrap(), Some(42));
    }

    // --- Test shutdown mentre si è in attesa ---

    #[test]
    fn test_shutdown_sblocca_recv_in_attesa() {
        let ch: MpMcChannel<i32> = MpMcChannel::new(4);
        let ch2 = ch.clone();

        let consumer = thread::spawn(move || {
            ch2.recv() // buffer vuoto: si blocca
        });

        thread::sleep(Duration::from_millis(50));
        ch.shutdown();
        // il consumer deve svegliarsi e ritornare None
        assert_eq!(consumer.join().unwrap(), None);
    }

    #[test]
    fn test_shutdown_sblocca_send_in_attesa() {
        let ch = MpMcChannel::new(2);
        let ch2 = ch.clone();

        ch.send(1);
        ch.send(2); // buffer pieno

        let producer = thread::spawn(move || {
            ch2.send(3) // si blocca
        });

        thread::sleep(Duration::from_millis(50));
        ch.shutdown();
        // il producer deve svegliarsi e ritornare None
        assert_eq!(producer.join().unwrap(), None);
    }

    // --- Test multi produttori / multi consumatori ---

    #[test]
    fn test_multi_producer_multi_consumer() {
        use std::sync::{Arc, Mutex};

        let ch = MpMcChannel::new(8);
        let ricevuti = Arc::new(Mutex::new(vec![]));

        let producers: Vec<_> = (0..4).map(|i| {
            let ch2 = ch.clone();
            thread::spawn(move || {
                ch2.send(i).unwrap();
            })
        }).collect();

        let consumers: Vec<_> = (0..4).map(|_| {
            let ch2 = ch.clone();
            let ric = Arc::clone(&ricevuti);
            thread::spawn(move || {
                let val = ch2.recv().unwrap();
                ric.lock().unwrap().push(val);
            })
        }).collect();

        for p in producers { p.join().unwrap(); }
        for c in consumers { c.join().unwrap(); }

        let mut ric = ricevuti.lock().unwrap();
        ric.sort();
        assert_eq!(*ric, vec![0, 1, 2, 3]);
    }
}