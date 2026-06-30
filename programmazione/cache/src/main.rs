use std::{collections::HashMap, sync::{Arc, Condvar, Mutex}};
use std::hash::Hash;

/*
Un componente con funzionalità di cache permette di ottimizzare il comportamento di un sistema 
riducendo il numero di volte in cui una funzione è invocata, tenendo traccia dei risultati da essa 
restituiti a fronte di un particolare dato in ingresso. Per generalità, si assuma che la funzione 
accetti un dato di tipo generico K e restituisca un valore di tipo generico V.

Il componente offre un unico metodo get(...) che prende in ingresso due parametri, 
il valore k (di tipo K, clonabile) del parametro e la funzione f (di tipo K -> V) 
responsabile della sua trasformazione, e restituisce uno smart pointer clonabile al relativo valore.

Se, per una determinata chiave k, non è ancora stato calcolato il valore corrispondente, 
la funzione viene invocata e ne viene restituito il risultato; altrimenti viene restituito il risultato già trovato.

Il componente cache deve essere thread-safe perché due o più thread 
possono richiedere contemporaneamente il valore di una data chiave: quando questo 
avviene e il dato non è ancora presente, la chiamata alla funzione dovrà essere eseguita 
nel contesto di UN SOLO thread, mentre gli altri dovranno aspettare il risultato in corso di elaborazione, 
SENZA CONSUMARE cicli macchina.

Si implementi tale componente a scelta nei linguaggi Rust.

*/

// =============== Inserisci implementazione



// =============== TEST


fn main() {
    println!("Hello, world!");
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
 
    // Test 1: funzionamento base
    #[test]
    fn test_get_base() {
        let cache = Cache::new();
        let result = cache.get(5, |k| k * 10);
        assert_eq!(*result, 50);
    }
 
    // Test 2: la stessa chiave restituisce lo stesso Arc (identico, non solo uguale)
    #[test]
    fn test_stessa_chiave_stesso_arc() {
        let cache = Cache::new();
        let r1 = cache.get(42, |k| k * 2);
        let r2 = cache.get(42, |k| k * 2);
        assert!(Arc::ptr_eq(&r1, &r2), "devono essere lo stesso Arc");
    }
 
    // Test 3: f viene chiamata UNA SOLA VOLTA per la stessa chiave
    #[test]
    fn test_f_chiamata_una_sola_volta() {
        let counter = Arc::new(Mutex::new(0u32));
        let cache = Cache::new();
 
        for _ in 0..5 {
            let cnt = counter.clone();
            cache.get(99, move |k| {
                *cnt.lock().unwrap() += 1;
                k * 2
            });
        }
 
        assert_eq!(*counter.lock().unwrap(), 1, "f deve essere chiamata una sola volta");
    }
 
    // Test 4: chiavi diverse producono valori diversi (f chiamata una volta per chiave)
    #[test]
    fn test_chiavi_diverse() {
        let counter = Arc::new(Mutex::new(0u32));
        let cache = Cache::new();
 
        for i in 0..3u32 {
            let cnt = counter.clone();
            let result = cache.get(i, move |k| {
                *cnt.lock().unwrap() += 1;
                k * 10
            });
            assert_eq!(*result, i * 10);
        }
 
        assert_eq!(*counter.lock().unwrap(), 3, "f chiamata una volta per ogni chiave");
    }
 
    // Test 5: thread-safety — N thread richiedono la stessa chiave contemporaneamente
    // f deve essere chiamata UNA SOLA VOLTA, tutti i thread ottengono il valore corretto
    #[test]
    fn test_concorrenza_stessa_chiave() {
        let cache = Arc::new(Cache::new());
        let counter = Arc::new(Mutex::new(0u32));
        let mut handles = vec![];
 
        for _ in 0..10 {
            let cache = cache.clone();
            let cnt = counter.clone();
 
            let h = thread::spawn(move || {
                cache.get("chiave", move |k| {
                    // Simula un calcolo lento
                    thread::sleep(Duration::from_millis(50));
                    *cnt.lock().unwrap() += 1;
                    format!("valore di {}", k)
                })
            });
            handles.push(h);
        }
 
        let results: Vec<Arc<String>> = handles.into_iter().map(|h| h.join().unwrap()).collect();
 
        // f chiamata esattamente una volta
        assert_eq!(*counter.lock().unwrap(), 1, "f deve essere chiamata una sola volta anche con N thread");
 
        // tutti i thread hanno ricevuto lo stesso Arc
        for r in &results {
            assert!(Arc::ptr_eq(r, &results[0]), "tutti devono avere lo stesso Arc");
        }
    }
 
    // Test 6: thread-safety — N thread, chiavi diverse, nessuna interferenza
    #[test]
    fn test_concorrenza_chiavi_diverse() {
        let cache = Arc::new(Cache::new());
        let mut handles = vec![];
 
        for i in 0..5u32 {
            let cache = cache.clone();
            let h = thread::spawn(move || {
                let result = cache.get(i, |k| k * 100);
                assert_eq!(*result, i * 100);
            });
            handles.push(h);
        }
 
        for h in handles {
            h.join().unwrap();
        }
    }
}
