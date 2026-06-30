use std::sync::{Arc, Condvar, Mutex};

/*
Una barriera è un costrutto di sincronizzazione usato per regolare l'avanzamento relativo della 
computazione di più thread. All'atto della costruzione di questo oggetto, viene indicato il numero N di thread coinvolti.

Non è lecito creare una barriera che coinvolga meno di 2 thread.

La barriera offre un solo metodo, wait(), il cui scopo è bloccare temporaneamente l'esecuzione del thread 
che lo ha invocato, non ritornando fino a che non sono giunte altre N-1 invocazioni dello stesso metodo da parte di 
altri thread: quando ciò succede, la barriera si sblocca e tutti tornano. 
Successive invocazioni del metodo wait() hanno lo stesso comportamento: la barriera è ciclica.

Attenzione a non mescolare le fasi di ingresso e di uscita!

Una RankingBarrier è una versione particolare della barriera in cui il metodo wait() restituisce un intero che 
rappresenta l'ordine di arrivo: il primo thread ad avere invocato wait() otterrà 1 come valore di ritorno, 
il secondo thread 2, e così via. All'inizio di un nuovo ciclo, il conteggio ripartirà da 1.

Si implementi la struttura dati RankingBarrier a scelta nei linguaggi Rust.
*/

// ============ Inserisci Implementazione: 






// =========== Test

fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use std::sync::atomic::{AtomicBool, Ordering};

    // --- Test costruzione ---

    #[test]
    fn test_costruzione_valida() {
        let _ = RankingBarrier::new(2);
        let _ = RankingBarrier::new(10);
    }

    #[test]
    #[should_panic]
    fn test_costruzione_invalida_1_thread() {
        RankingBarrier::new(1);
    }

    #[test]
    #[should_panic]
    fn test_costruzione_invalida_0_thread() {
        RankingBarrier::new(0);
    }

    // --- Test rank ---

    #[test]
    fn test_rank_contiene_tutti_valori() {
        let n = 4;
        let barrier = Arc::new(RankingBarrier::new(n));
        let risultati = Arc::new(Mutex::new(vec![]));

        let handles: Vec<_> = (0..n).map(|_| {
            let b = Arc::clone(&barrier);
            let r = Arc::clone(&risultati);
            thread::spawn(move || {
                let rank = b.wait();
                r.lock().unwrap().push(rank);
            })
        }).collect();

        for h in handles { h.join().unwrap(); }

        let mut r = risultati.lock().unwrap();
        r.sort();
        assert_eq!(*r, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_rank_tutti_unici() {
        let n = 5;
        let barrier = Arc::new(RankingBarrier::new(n));
        let risultati = Arc::new(Mutex::new(vec![]));

        let handles: Vec<_> = (0..n).map(|_| {
            let b = Arc::clone(&barrier);
            let r = Arc::clone(&risultati);
            thread::spawn(move || {
                let rank = b.wait();
                r.lock().unwrap().push(rank);
            })
        }).collect();

        for h in handles { h.join().unwrap(); }

        let r = risultati.lock().unwrap();
        let unique: std::collections::HashSet<_> = r.iter().collect();
        assert_eq!(unique.len(), n); // nessun rank duplicato
    }

    // --- Test blocco ---

    #[test]
    fn test_thread_si_blocca_finche_non_arrivano_tutti() {
        let barrier = Arc::new(RankingBarrier::new(2));
        let passato = Arc::new(AtomicBool::new(false));

        let b = Arc::clone(&barrier);
        let p = Arc::clone(&passato);

        let h = thread::spawn(move || {
            b.wait();
            p.store(true, Ordering::SeqCst);
        });

        thread::sleep(Duration::from_millis(100));
        assert!(
            !passato.load(Ordering::SeqCst),
            "il thread non deve passare prima che arrivino N thread"
        );

        barrier.wait(); // sblocca
        h.join().unwrap();

        assert!(passato.load(Ordering::SeqCst));
    }

    // --- Test ciclicità ---

    #[test]
    fn test_cicli_multipli_rank_corretti() {
        let n = 3;
        let num_cicli = 5;
        let barrier = Arc::new(RankingBarrier::new(n));

        let handles: Vec<_> = (0..n).map(|_| {
            let b = Arc::clone(&barrier);
            thread::spawn(move || {
                (0..num_cicli).map(|_| b.wait()).collect::<Vec<_>>()
            })
        }).collect();

        let tutti: Vec<Vec<usize>> = handles.into_iter()
            .map(|h| h.join().unwrap())
            .collect();

        // in ogni ciclo i rank devono essere esattamente {1, 2, 3}
        for ciclo in 0..num_cicli {
            let mut ranks: Vec<usize> = tutti.iter().map(|r| r[ciclo]).collect();
            ranks.sort();
            assert_eq!(ranks, vec![1, 2, 3], "ciclo {} ha rank sbagliati", ciclo + 1);
        }
    }

    #[test]
    fn test_ciclo_riparte_da_1() {
        let n = 2;
        let barrier = Arc::new(RankingBarrier::new(n));

        let b = Arc::clone(&barrier);
        let h = thread::spawn(move || {
            let r1 = b.wait();
            let r2 = b.wait();
            (r1, r2)
        });

        let r1_main = barrier.wait();
        let r2_main = barrier.wait();
        let (r1_thread, r2_thread) = h.join().unwrap();

        // ogni ciclo deve contenere rank 1 e 2
        let mut ciclo1 = vec![r1_main, r1_thread];
        ciclo1.sort();
        assert_eq!(ciclo1, vec![1, 2]);

        let mut ciclo2 = vec![r2_main, r2_thread];
        ciclo2.sort();
        assert_eq!(ciclo2, vec![1, 2], "il secondo ciclo deve ripartire da 1");
    }

    // --- Test no mescolanza fasi ---

    #[test]
    fn test_no_mescolanza_fasi() {
        // Thread veloce entra subito nel ciclo 2 dopo il ciclo 1.
        // Thread lento aspetta prima di entrare nel ciclo 2.
        // I rank dei due cicli devono essere indipendenti.
        let n = 2;
        let barrier = Arc::new(RankingBarrier::new(n));

        let b1 = Arc::clone(&barrier);
        let b2 = Arc::clone(&barrier);

        let h1 = thread::spawn(move || {
            let r1 = b1.wait();
            let r2 = b1.wait(); // entra subito nel ciclo 2
            (r1, r2)
        });

        let h2 = thread::spawn(move || {
            let r1 = b2.wait();
            thread::sleep(Duration::from_millis(50)); // lento ad uscire dal ciclo 1
            let r2 = b2.wait();
            (r1, r2)
        });

        let (r1_t1, r2_t1) = h1.join().unwrap();
        let (r1_t2, r2_t2) = h2.join().unwrap();

        let mut ciclo1 = vec![r1_t1, r1_t2];
        ciclo1.sort();
        assert_eq!(ciclo1, vec![1, 2]);

        let mut ciclo2 = vec![r2_t1, r2_t2];
        ciclo2.sort();
        assert_eq!(ciclo2, vec![1, 2]);
    }

    // --- Test N = 2 base ---

    #[test]
    fn test_due_thread_base() {
        let barrier = Arc::new(RankingBarrier::new(2));
        let b = Arc::clone(&barrier);

        let h = thread::spawn(move || b.wait());
        let r_main = barrier.wait();
        let r_thread = h.join().unwrap();

        let mut ranks = vec![r_main, r_thread];
        ranks.sort();
        println!("{:?}", ranks);
        assert_eq!(ranks, vec![1, 2]);
    }
}