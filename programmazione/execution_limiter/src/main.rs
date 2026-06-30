use std::sync::{Arc, Condvar, Mutex};

/*
ExecutionLimiter

All'interno di un programma è necessario garantire che non vengano eseguite CONTEMPORANEAMENTE più di N invocazioni di
operazioni potenzialmente lente.
A questo scopo, è stata definita la struttura dati ExecutionLimiter che
viene inizializzata con il valore N del limite. Tale struttura è thread-safe e offre solo il metodo pubblico
generico execute( f ), che accetta come unico parametro una funzione f, priva di parametri che ritorna il
tipo generico R. Il metodo execute(...) ha, come tipo di ritorno, lo stesso tipo R restituito da f ed
ha il compito di mantere il conteggio di quante invocazioni sono in corso. Se tale numero è già pari al
valore N definito all'atto della costruzione della struttura dati, attende, senza provocare consumo di CPU,
che scenda sotto soglia, dopodiché invoca la funzione f ricevuta come parametro e ne restituisce il valore.
Poiché l'esecuzione della funzione f potrebbe fallire, in tale caso, si preveda di decrementare il conteggio correttamente.

Si implementi, usando il linguaggio Rust, tale struttura dati, garantendo tutte le funzionalità richieste."
*/





// =========== Inserisci Implementazione: 



// ========== Test

fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier, Mutex};
    use std::thread;
    use std::time::Duration;

    // ── 1. Funzionalità di base ──────────────────────────────────────────────

    #[test]
    fn test_ritorna_valore_corretto() {
        let lim = ExecutionLimiter::new(2);
        assert_eq!(lim.execute(|| 42), 42);
    }

    #[test]
    fn test_ritorna_stringa() {
        let lim = ExecutionLimiter::new(2);
        assert_eq!(lim.execute(|| "ciao".to_string()), "ciao");
    }

    #[test]
    fn test_esecuzioni_sequenziali() {
        let lim = ExecutionLimiter::new(1);
        for i in 0..10_usize {
            assert_eq!(lim.execute(|| i * 3), i * 3);
        }
    }

    // ── 2. Rispetto del limite N ─────────────────────────────────────────────

    #[test]
    fn test_concorrenza_massima_rispettata() {
        let n = 3_usize;
        let lim = Arc::new(ExecutionLimiter::new(n));
        // (current_concurrent, max_seen)
        let state = Arc::new(Mutex::new((0_usize, 0_usize)));
        let num_tasks = 12;
        let barrier = Arc::new(Barrier::new(num_tasks));
        let mut handles = vec![];

        for _ in 0..num_tasks {
            let lim = Arc::clone(&lim);
            let state = Arc::clone(&state);
            let barrier = Arc::clone(&barrier);

            handles.push(thread::spawn(move || {
                barrier.wait(); // tutti partono contemporaneamente
                lim.execute(|| {
                    {
                        let mut s = state.lock().unwrap();
                        s.0 += 1;
                        s.1 = s.1.max(s.0);
                    }
                    thread::sleep(Duration::from_millis(20));
                    state.lock().unwrap().0 -= 1;
                });
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        let (_, max_seen) = *state.lock().unwrap();
        assert!(max_seen <= n, "Limite violato: {max_seen} > {n}");
    }

    #[test]
    fn test_n1_serializza_completamente() {
        // Con N=1 i task non devono mai sovrapporsi:
        // il log deve avere coppie consecutive con lo stesso tag (enter/exit).
        let lim = Arc::new(ExecutionLimiter::new(1));
        let log: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
        let barrier = Arc::new(Barrier::new(3));
        let mut handles = vec![];

        for tag in ["A", "B", "C"] {
            let lim = Arc::clone(&lim);
            let log = Arc::clone(&log);
            let barrier = Arc::clone(&barrier);

            handles.push(thread::spawn(move || {
                barrier.wait();
                lim.execute(|| {
                    log.lock().unwrap().push(tag); // enter
                    thread::sleep(Duration::from_millis(10));
                    log.lock().unwrap().push(tag); // exit
                });
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        let log = log.lock().unwrap();
        assert_eq!(log.len(), 6);
        for chunk in log.chunks(2) {
            assert_eq!(chunk[0], chunk[1], "Task sovrapposti rilevati: {log:?}");
        }
    }

    // ── 3. Completamento garantito ───────────────────────────────────────────

    #[test]
    fn test_tutti_i_task_completano() {
        let lim = Arc::new(ExecutionLimiter::new(2));
        let completed = Arc::new(Mutex::new(0_usize));
        let num_tasks = 10;
        let mut handles = vec![];

        for _ in 0..num_tasks {
            let lim = Arc::clone(&lim);
            let completed = Arc::clone(&completed);

            handles.push(thread::spawn(move || {
                lim.execute(|| {
                    thread::sleep(Duration::from_millis(15));
                    *completed.lock().unwrap() += 1;
                });
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(*completed.lock().unwrap(), num_tasks);
    }

    // ── 4. Panic safety ──────────────────────────────────────────────────────

    #[test]
    fn test_panic_decrementa_contatore() {
        // Dopo un panic dentro execute() il limiter non deve bloccarsi.
        let lim = Arc::new(ExecutionLimiter::new(1));
        let lim2 = Arc::clone(&lim);

        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            lim.execute(|| -> i32 { panic!("panic intenzionale") });
        }));

        // Se il contatore non venisse decrementato qui andrebbe in deadlock
        assert_eq!(lim2.execute(|| 99), 99);
    }

    #[test]
    fn test_n_panic_consecutivi_non_bloccano() {
        let n = 3_usize;
        let lim = Arc::new(ExecutionLimiter::new(n));

        for _ in 0..n {
            let lim = Arc::clone(&lim);
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                lim.execute(|| -> i32 { panic!("boom") });
            }));
        }

        // Il limiter deve essere ancora operativo
        assert_eq!(lim.execute(|| 7), 7);
    }

    #[test]
    fn test_panic_misto_a_task_normali() {
        // Alcune goroutine falliscono, altre no: tutte devono completare.
        let lim = Arc::new(ExecutionLimiter::new(2));
        let completed = Arc::new(Mutex::new(0_usize));
        let mut handles = vec![];

        for i in 0..8_usize {
            let lim = Arc::clone(&lim);
            let completed = Arc::clone(&completed);

            handles.push(thread::spawn(move || {
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                    lim.execute(|| {
                        thread::sleep(Duration::from_millis(10));
                        if i % 3 == 0 {
                            panic!("task {i} fallisce");
                        }
                        *completed.lock().unwrap() += 1;
                    });
                }));
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // i task che non hanno panicato (i ∉ {0,3,6}) sono 5
        assert_eq!(*completed.lock().unwrap(), 5);
    }
}