use std::{sync::{Arc, Condvar, Mutex}, time::{ Instant}};


/*
    Una DelayedQueue<T:Send> è un particolare tipo di coda non limitata che offre tre metodi principali, 
    oltre alla funzione costruttrice: 
    
    1. offer(&self, t:T, i: Instant) : Inserisce un elemento che non 
    potrà essere estratto prima dell'istante di scadenza i. 
    
    2. take(&self) -> Option: Cerca l'elemento t 
    con scadenza più ravvicinata: se tale scadenza è già stata oltrepassata, restituisce Some(t); 
    se la scadenza non è ancora stata superata, attende senza consumare cicli di CPU, che tale tempo trascorra, 
    per poi restituire Some(t); se non è presente nessun elemento in coda, restituisce None. 
    Se, durante l'attesa, avviene un cambiamento qualsiasi al contenuto della coda, 
    ripete il procedimento suddetto con il nuovo elemento a scadenza più ravvicinata (ammesso che ci sia ancora). 
    
    3. size(&self) -> usize: restituisce il numero di elementi in coda indipendentemente dal fatto che siano scaduti o meno. 
    
    Si implementi tale struttura dati nel linguaggio Rust, avendo cura di renderne il comportamento thread-safe. 
    Si ricordi che gli oggetti di tipo Condvar offrono un meccanismo di attesa limitata nel tempo, 
    offerto dai metodi wait_timeout(...) e wait_timeout_while(...).
*/


// ========= Implementazione


// ========= TEST

fn main() {
    println!("Hello, world!");
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

    // ── Coda vuota ──────────────────────────────────────────────────────────

    #[test]
    fn test_take_coda_vuota_restituisce_none() {
        let q: DelayedQueue<i32> = DelayedQueue::new();
        assert_eq!(q.take(), None);
    }

    #[test]
    fn test_size_coda_vuota_e_zero() {
        let q: DelayedQueue<i32> = DelayedQueue::new();
        assert_eq!(q.size(), 0);
    }

    // ── offer / size ─────────────────────────────────────────────────────────

    #[test]
    fn test_offer_aumenta_size() {
        let q: DelayedQueue<i32> = DelayedQueue::new();
        q.offer(1, Instant::now() + Duration::from_secs(60));
        assert_eq!(q.size(), 1);
        q.offer(2, Instant::now() + Duration::from_secs(60));
        assert_eq!(q.size(), 2);
    }

    #[test]
    fn test_size_conta_tutti_scaduti_e_non() {
        // size() deve contare tutti gli elementi indipendentemente dalla scadenza
        let q: DelayedQueue<i32> = DelayedQueue::new();
        q.offer(1, Instant::now() - Duration::from_secs(1)); // già scaduto
        q.offer(2, Instant::now() + Duration::from_secs(60)); // non ancora scaduto
        assert_eq!(q.size(), 2);
    }

    #[test]
    fn test_size_decresce_dopo_ogni_take() {
        let q: DelayedQueue<i32> = DelayedQueue::new();
        q.offer(1, Instant::now() - Duration::from_secs(1));
        q.offer(2, Instant::now() - Duration::from_secs(1));
        assert_eq!(q.size(), 2);

        q.take();
        assert_eq!(q.size(), 1);

        q.take();
        assert_eq!(q.size(), 0);

        // coda di nuovo vuota → None
        assert_eq!(q.take(), None);
    }

    // ── take: comportamento temporale ────────────────────────────────────────

    #[test]
    fn test_take_elemento_gia_scaduto_ritorna_subito() {
        let q: DelayedQueue<i32> = DelayedQueue::new();
        q.offer(42, Instant::now() - Duration::from_secs(1));

        let start = Instant::now();
        let result = q.take();
        let elapsed = start.elapsed();

        assert_eq!(result, Some(42));
        assert!(
            elapsed < Duration::from_millis(50),
            "Attesa inattesa di {:?} per un elemento già scaduto",
            elapsed
        );
    }

    #[test]
    fn test_take_attende_elemento_futuro_e_poi_lo_restituisce() {
        let q: DelayedQueue<i32> = DelayedQueue::new();
        let delay = Duration::from_millis(300);
        q.offer(99, Instant::now() + delay);

        let start = Instant::now();
        let result = q.take();
        let elapsed = start.elapsed();

        assert_eq!(result, Some(99));
        assert!(
            elapsed >= delay,
            "Ha restituito l'elemento prima della scadenza ({:?} < {:?})",
            elapsed, delay
        );
        assert!(
            elapsed < delay + Duration::from_millis(300),
            "Attesa eccessiva: {:?}",
            elapsed
        );
    }

    // ── Ordinamento per scadenza ─────────────────────────────────────────────

    #[test]
    fn test_take_restituisce_scadenza_piu_ravvicinata() {
        let q: DelayedQueue<i32> = DelayedQueue::new();
        let now = Instant::now();

        q.offer(10, now + Duration::from_millis(500));
        q.offer(20, now - Duration::from_millis(100)); // scaduto prima di tutti
        q.offer(30, now + Duration::from_millis(200));

        let result = q.take();
        assert_eq!(result, Some(20));
        assert_eq!(q.size(), 2);
    }

    #[test]
    fn test_take_estrae_in_ordine_crescente_di_scadenza() {
        let q: DelayedQueue<&str> = DelayedQueue::new();
        let now = Instant::now();

        // Inseriti in ordine casuale rispetto alla scadenza
        q.offer("C", now - Duration::from_millis(10));
        q.offer("A", now - Duration::from_millis(100)); // scade prima
        q.offer("B", now - Duration::from_millis(50));

        assert_eq!(q.take(), Some("A"));
        assert_eq!(q.take(), Some("B"));
        assert_eq!(q.take(), Some("C"));
        assert_eq!(q.take(), None); // coda ora vuota
    }

    // ── Risveglio anticipato su modifica della coda ──────────────────────────

    #[test]
    fn test_take_si_risveglia_se_arriva_elemento_piu_urgente() {
        // Scenario:
        //   Thread principale chiama take() → trova un elemento con scadenza tra 2s
        //   Dopo 150ms un altro thread inserisce un elemento già scaduto
        //   take() deve svegliarsi, ricominciare il procedimento e tornare subito
        let q = Arc::new(DelayedQueue::new());
        let q2 = Arc::clone(&q);

        q.offer(1i32, Instant::now() + Duration::from_secs(2));

        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(150));
            q2.offer(2i32, Instant::now() - Duration::from_millis(1));
        });

        let start = Instant::now();
        let result = q.take();
        let elapsed = start.elapsed();

        handle.join().unwrap();

        assert_eq!(result, Some(2), "Deve restituire l'elemento con scadenza più ravvicinata");
        assert!(
            elapsed < Duration::from_millis(800),
            "Doveva svegliarsi prima della scadenza originale (2s), invece ha atteso {:?}",
            elapsed
        );
    }

    #[test]
    fn test_take_attende_se_coda_vuota_poi_arriva_elemento() {
        // take() è bloccante solo se c'è almeno un elemento, altrimenti None immediato.
        // Qui verifichiamo che su coda vuota torni None senza bloccarsi.
        let q: DelayedQueue<i32> = DelayedQueue::new();

        let start = Instant::now();
        let result = q.take();
        let elapsed = start.elapsed();

        assert_eq!(result, None);
        assert!(
            elapsed < Duration::from_millis(50),
            "take() su coda vuota non deve bloccarsi, ha atteso {:?}",
            elapsed
        );
    }

    // ── Thread-safety ─────────────────────────────────────────────────────────

    #[test]
    fn test_offer_concorrenti_incrementano_size_correttamente() {
        let q = Arc::new(DelayedQueue::new());
        let mut handles = vec![];

        for i in 0..20i32 {
            let q_clone = Arc::clone(&q);
            handles.push(thread::spawn(move || {
                q_clone.offer(i, Instant::now() - Duration::from_millis(1));
            }));
        }
        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(q.size(), 20);
    }

    #[test]
    fn test_take_concorrenti_nessun_duplicato_e_nessuna_perdita() {
        let q = Arc::new(DelayedQueue::new());

        // Tutti già scaduti
        for i in 0..10i32 {
            q.offer(i, Instant::now() - Duration::from_millis(1));
        }

        let results = Arc::new(Mutex::new(Vec::new()));
        let mut handles = vec![];

        for _ in 0..10 {
            let q_clone = Arc::clone(&q);
            let results_clone = Arc::clone(&results);
            handles.push(thread::spawn(move || {
                if let Some(v) = q_clone.take() {
                    results_clone.lock().unwrap().push(v);
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }

        let mut results = results.lock().unwrap().clone();
        results.sort();
        // Tutti e 10 estratti, senza duplicati
        assert_eq!(results.len(), 10, "Devono essere estratti tutti e 10 gli elementi");
        results.dedup();
        assert_eq!(results.len(), 10, "Non ci devono essere duplicati");
    }

    #[test]
    fn test_producer_consumer_multithread() {
        let q = Arc::new(DelayedQueue::new());
        let q_prod = Arc::clone(&q);

        // Il producer inserisce 5 elementi con scadenze crescenti
        let producer = thread::spawn(move || {
            for i in 0..5i32 {
                q_prod.offer(i, Instant::now() + Duration::from_millis(i as u64 * 60));
                thread::sleep(Duration::from_millis(10));
            }
        });
        producer.join().unwrap();

        let mut extracted = Vec::new();
        while q.size() > 0 {
            if let Some(v) = q.take() {
                extracted.push(v);
            }
        }

        assert_eq!(extracted.len(), 5);
        // L'ordine di estrazione deve seguire le scadenze crescenti
        assert_eq!(extracted, vec![0, 1, 2, 3, 4]);
    }
}