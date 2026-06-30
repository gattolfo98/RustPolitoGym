use std::sync::{Arc, Condvar, Mutex};


/*

La classe generica Exchanger permette a due thread di scambiarsi un valore di tipo T. 
Essa offre esclusivamente il metodo pubblico T exchange( T t) che blocca il thread chiamante senza 
consumare CPU fino a che un altro thread non invoca lo stesso metodo, sulla stessa istanza. 
Quando questo avviene, il metodo restituisce l’oggetto passato come parametro dal thread opposto.
*/


// ========== Implementazione 




// ========= Test 


fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    // ── Correttezza dello scambio ────────────────────────────────────────────

    #[test]
    fn test_scambio_base_interi() {
        let ex = Arc::new(Exchanger::new());
        let ex2 = Arc::clone(&ex);

        let handle = thread::spawn(move || ex2.exchange(42i32));

        let ricevuto_main   = ex.exchange(99i32);
        let ricevuto_thread = handle.join().unwrap();

        assert_eq!(ricevuto_main,   42);   // main riceve il valore del thread
        assert_eq!(ricevuto_thread, 99);   // thread riceve il valore di main
    }

    #[test]
    fn test_scambio_string_ownership() {
        let ex = Arc::new(Exchanger::new());
        let ex2 = Arc::clone(&ex);

        let handle = thread::spawn(move || ex2.exchange(String::from("dal thread")));

        let ricevuto = ex.exchange(String::from("dal main"));
        assert_eq!(ricevuto, "dal thread");
        assert_eq!(handle.join().unwrap(), "dal main");
    }

    #[test]
    fn test_scambio_struct_custom() {
        #[derive(Debug, PartialEq)]
        struct Pacchetto { id: u32, msg: String }

        let ex = Arc::new(Exchanger::new());
        let ex2 = Arc::clone(&ex);

        let handle = thread::spawn(move || {
            ex2.exchange(Pacchetto { id: 2, msg: "thread".into() })
        });

        let ricevuto = ex.exchange(Pacchetto { id: 1, msg: "main".into() });

        assert_eq!(ricevuto.id,  2);
        assert_eq!(ricevuto.msg, "thread");
        let dal_thread = handle.join().unwrap();
        assert_eq!(dal_thread.id,  1);
        assert_eq!(dal_thread.msg, "main");
    }

    #[test]
    fn test_scambio_option() {
        let ex = Arc::new(Exchanger::<Option<i32>>::new());
        let ex2 = Arc::clone(&ex);

        let handle = thread::spawn(move || ex2.exchange(None));

        let ricevuto = ex.exchange(Some(42));
        assert_eq!(ricevuto, None);
        assert_eq!(handle.join().unwrap(), Some(42));
    }

    // ── Integrità: nessuna perdita né duplicazione ───────────────────────────

    #[test]
    fn test_nessuna_perdita_ne_duplicazione() {
        // La somma totale dei valori non deve cambiare dopo lo scambio
        let ex = Arc::new(Exchanger::new());
        let ex2 = Arc::clone(&ex);

        let handle = thread::spawn(move || ex2.exchange(1u64));

        let r_main   = ex.exchange(2u64);
        let r_thread = handle.join().unwrap();

        assert_eq!(r_main,   1u64, "main doveva ricevere 1");
        assert_eq!(r_thread, 2u64, "thread doveva ricevere 2");
        assert_eq!(r_main + r_thread, 3u64, "la somma deve essere conservata");
    }

    // ── Comportamento bloccante ──────────────────────────────────────────────

    #[test]
    fn test_primo_thread_attende_il_secondo() {
        let ex = Arc::new(Exchanger::new());
        let ex2 = Arc::clone(&ex);
        let ritardo = Duration::from_millis(300);

        // Il secondo thread arriva deliberatamente in ritardo
        let handle = thread::spawn(move || {
            thread::sleep(ritardo);
            ex2.exchange("secondo")
        });

        let start   = Instant::now();
        let ricevuto = ex.exchange("primo");
        let elapsed  = start.elapsed();

        handle.join().unwrap();

        assert_eq!(ricevuto, "secondo");
        assert!(
            elapsed >= ritardo - Duration::from_millis(30),
            "Il main non ha aspettato il secondo thread: {:?}", elapsed
        );
    }

    #[test]
    fn test_secondo_thread_si_blocca_se_arriva_prima() {
        let ex = Arc::new(Exchanger::new());
        let ex2 = Arc::clone(&ex);
        let ritardo = Duration::from_millis(300);

        // Questo thread parte subito e si blocca in attesa
        let handle = thread::spawn(move || ex2.exchange("arrivato prima"));

        // Il main arriva in ritardo: lo scambio deve avvenire quasi istantaneamente
        thread::sleep(ritardo);
        let start    = Instant::now();
        let ricevuto = ex.exchange("arrivato dopo");
        let elapsed  = start.elapsed();

        assert_eq!(ricevuto, "arrivato prima");
        assert_eq!(handle.join().unwrap(), "arrivato dopo");
        assert!(
            elapsed < Duration::from_millis(80),
            "Dopo l'arrivo del secondo thread lo scambio deve essere quasi immediato: {:?}",
            elapsed
        );
    }

    #[test]
    fn test_blocco_passivo_senza_consumo_cpu() {
        // Non misuriamo la CPU direttamente, ma verifichiamo che il thread
        // rimanga bloccato per l'intera durata attesa (no busy-wait).
        let ex = Arc::new(Exchanger::new());
        let ex2 = Arc::clone(&ex);
        let attesa = Duration::from_millis(400);

        let handle = thread::spawn(move || {
            thread::sleep(attesa);
            ex2.exchange(0i32)
        });

        let start   = Instant::now();
        ex.exchange(0i32);
        let elapsed = start.elapsed();

        handle.join().unwrap();

        assert!(elapsed >= attesa - Duration::from_millis(30),
            "Attesa troppo breve, possibile busy-wait: {:?}", elapsed);
        assert!(elapsed < attesa + Duration::from_millis(200),
            "Attesa eccessiva: {:?}", elapsed);
    }

    // ── Riutilizzo della stessa istanza ──────────────────────────────────────

    #[test]
    fn test_scambi_sequenziali_stessa_istanza() {
        // L'Exchanger deve funzionare correttamente per più scambi in sequenza
        let ex = Arc::new(Exchanger::new());

        for round in 0u32..5 {
            let ex2 = Arc::clone(&ex);
            let handle = thread::spawn(move || ex2.exchange(round * 10));

            let ricevuto = ex.exchange(round);

            assert_eq!(ricevuto, round * 10,
                "Round {}: main doveva ricevere {}", round, round * 10);
            assert_eq!(handle.join().unwrap(), round,
                "Round {}: thread doveva ricevere {}", round, round);
        }
    }

    // ── Concorrenza: N coppie in parallelo ───────────────────────────────────

    #[test]
    fn test_n_coppie_concorrenti_exchanger_dedicato() {
        // N coppie di thread, ognuna con il proprio Exchanger.
        // Garantisce che A_i scambi esclusivamente con B_i.
        let n = 8usize;
        let mut handles = vec![];

        for i in 0..n {
            let ex   = Arc::new(Exchanger::new());
            let ex_b = Arc::clone(&ex);

            // Thread A: invia i, si aspetta i + 100
            handles.push(thread::spawn(move || {
                let r = ex.exchange(i as i32);
                assert_eq!(r, (i + 100) as i32,
                    "Thread A[{}]: ricevuto {} invece di {}", i, r, i + 100);
            }));

            // Thread B: invia i + 100, si aspetta i
            handles.push(thread::spawn(move || {
                let r = ex_b.exchange((i + 100) as i32);
                assert_eq!(r, i as i32,
                    "Thread B[{}]: ricevuto {} invece di {}", i, r, i);
            }));
        }

        for h in handles {
            h.join().unwrap();
        }
    }
}