use std::{collections::HashMap, sync::{Arc, Condvar, Mutex}};
/*
In una macchina utensile, sono in esecuzione N thread concorrenti, 
ciascuno dei quali rileva continuamente una sequenza di valori, 
risultato dell'elaborazione delle misurazioni di un sensore. 
I valori devono essere raggruppati N a N in una struttura dati per essere ulteriormente trattati dal sistema. 
A questo scopo è definita la seguente classe thread-safe:

class Joiner {
    public: Joiner(int N); // N is the number of values that must be conferred
    std::map<int, double> supply(int key, double value);
};

Il metodo bloccante supply(...) riceve una coppia chiave/valore generata da un singolo thread e si 
blocca senza consumare CPU fino a che gli altri N-1 thread hanno inviato le loro misurazioni. 
Quando sono arrivate N misurazioni (corrispondenti ad altrettante invocazioni concorrenti), 
si sblocca e ciascuna invocazione precedentemente bloccata restituisce una mappa che contiene N elementi 
(uno per ciascun fornitore). Dopodiché, l'oggetto Joiner pulisce il proprio stato e si prepara ad 
accettare un nuovo gruppo di N misurazioni, in modo ciclico.

Si implementi tale classe, facendo attenzione a non mescolare nuovi conferimenti con quelli 
della tornata precedente (un thread appena uscito potrebbe essere molto veloce a rientrare, 
ripresentandosi con un nuovo valore quando lo stato non è ancora stato ripulito).

*/



// ========= Implementazione 




// ========= Test

fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Duration;

    /// Lancia `n` thread, ognuno fa supply(i, i * base), restituisce tutti i risultati.
    fn run_round(joiner: &Arc<Joiner>, n: usize, base: i128) -> Vec<HashMap<i64, i128>> {
        let handles: Vec<_> = (0..n)
            .map(|i| {
                let j = Arc::clone(joiner);
                thread::spawn(move || j.supply(i as i64, i as i128 * base))
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("thread panicked")).collect()
    }

    // --- Caso banale: N=1 ------------------------------------------------------
    #[test]
    fn test_n1_trivial() {
        let j = Arc::new(Joiner::new(1));
        let result = j.supply(42, 99);
        assert_eq!(result.len(), 1);
        assert_eq!(result[&42], 99);
    }

    // --- La mappa contiene esattamente N entry con i valori corretti ------------
    #[test]
    fn test_result_completeness() {
        let n = 5;
        let j = Arc::new(Joiner::new(n));
        let results = run_round(&j, n, 10);

        let expected: HashMap<i64, i128> =
            (0..n).map(|i| (i as i64, i as i128 * 10)).collect();

        for result in &results {
            assert_eq!(*result, expected);
        }
    }

    // --- Tutti i thread ricevono la stessa mappa identica ----------------------
    #[test]
    fn test_all_threads_same_result() {
        let n = 6;
        let j = Arc::new(Joiner::new(n));
        let results = run_round(&j, n, 1);

        for result in &results {
            assert_eq!(results[0], *result);
        }
    }

    // --- Dopo un ciclo il Joiner si resetta e accetta un nuovo gruppo ----------
    #[test]
    fn test_multiple_rounds() {
        let n = 4;
        let j = Arc::new(Joiner::new(n));

        for round in 1..=10i128 {
            let results = run_round(&j, n, round);
            assert_eq!(results[0].len(), n, "round {round}: dimensione mappa errata");
            for r in &results {
                assert_eq!(results[0], *r, "round {round}: risultati discordanti");
            }
        }
    }

    // --- CASO CRITICO: il thread veloce che rientra subito non mescola i dati --
    //
    // Thread 0 (veloce): supply(0, 0) → supply(0, 1000)  senza aspettare nessuno
    // Thread 1..n-1 (lenti): supply nel ciclo 1, poi Barrier, poi ciclo 2
    //
    // Invariante attesa:
    //   ciclo 1 → tutti i valori < 1000  (nessun dato del ciclo 2 trapela)
    //   ciclo 2 → tutti i valori >= 1000 (nessun dato del ciclo 1 rimane)
    #[test]
    fn test_no_mixing_on_fast_reentry() {
        let n = 4;
        let j = Arc::new(Joiner::new(n));
        let barrier = Arc::new(Barrier::new(n - 1));

        let j0 = Arc::clone(&j);
        let fast = thread::spawn(move || {
            let r1 = j0.supply(0, 0);
            let r2 = j0.supply(0, 1000); // tenta di rientrare immediatamente
            (r1, r2)
        });

        let slow_handles: Vec<_> = (1..n)
            .map(|i| {
                let j = Arc::clone(&j);
                let b = Arc::clone(&barrier);
                thread::spawn(move || {
                    let r1 = j.supply(i as i64, i as i128);
                    b.wait();
                    let r2 = j.supply(i as i64, i as i128 + 1000);
                    (r1, r2)
                })
            })
            .collect();

        let (fast_r1, fast_r2) = fast.join().expect("fast thread panicked");
        let slow_results: Vec<_> = slow_handles
            .into_iter()
            .map(|h| h.join().expect("slow thread panicked"))
            .collect();

        // Ciclo 1: valori tutti < 1000, mappa uguale per tutti
        assert_eq!(fast_r1.len(), n);
        for (_, v) in &fast_r1 {
            assert!(*v < 1000, "ciclo 1 contaminato da dati del ciclo 2: v={v}");
        }
        for (r1, _) in &slow_results {
            assert_eq!(*r1, fast_r1, "ciclo 1: risultati inconsistenti tra thread");
        }

        // Ciclo 2: valori tutti >= 1000, mappa uguale per tutti
        assert_eq!(fast_r2.len(), n);
        for (_, v) in &fast_r2 {
            assert!(*v >= 1000, "ciclo 2 contaminato da dati del ciclo 1: v={v}");
        }
        for (_, r2) in &slow_results {
            assert_eq!(*r2, fast_r2, "ciclo 2: risultati inconsistenti tra thread");
        }
    }

    // --- Il thread veloce si blocca davvero finché non arrivano tutti N --------
    //
    // Verificato indirettamente: se supply tornasse prima del tempo,
    // la mappa avrebbe meno di N entry o mancherebbe la chiave del thread lento.
    #[test]
    fn test_blocks_until_all_n_supplied() {
        let n = 4;
        let j = Arc::new(Joiner::new(n));

        let j_slow = Arc::clone(&j);
        let slow = thread::spawn(move || {
            thread::sleep(Duration::from_millis(200));
            j_slow.supply(0i64, 999i128)
        });

        let fast_handles: Vec<_> = (1..n)
            .map(|i| {
                let jj = Arc::clone(&j);
                thread::spawn(move || jj.supply(i as i64, i as i128))
            })
            .collect();

        let mut all = vec![slow.join().unwrap()];
        for h in fast_handles {
            all.push(h.join().unwrap());
        }

        for r in &all {
            assert_eq!(r.len(), n, "mappa incompleta: il thread non si è bloccato");
            assert_eq!(r[&0], 999, "chiave del thread lento assente o errata");
        }
    }

    // --- Stress: tanti thread, tanti cicli -------------------------------------
    #[test]
    fn test_stress() {
        let n = 8;
        let j = Arc::new(Joiner::new(n));

        for round in 1..=30i128 {
            let results = run_round(&j, n, round);
            assert_eq!(results[0].len(), n);
            for r in &results {
                assert_eq!(results[0], *r);
            }
        }
    }
}