# Domande sulla programmazione asincrona 

## Generate con IA

### 1. Qual è la differenza concettuale tra usare i thread e usare l'esecuzione asincrona?
### 2. Nell'event loop, quali sono i tre passi che vengono ripetuti in ciclo?
### 3. Cosa si intende per "callback hell", e perché è un problema (oltre alla semplice leggibilità)?
### 4. Nella versione con i Future `(.and_then(...))`, cosa rappresenta concretamente `and_then`? Che differenza c'è rispetto a passare una callback diretta come si faceva prima?
### 5. Cosa significa dire che `Poll::Pending` viene restituito da `poll()`? E cosa succede subito dopo, concettualmente, perché la macchina a stati possa "ripartire"?
### 6. Quando scrivi:
```rust 
async fn copy(file1: String, file2: String) -> Result<()> { ... }
```
Che tipo ha davvero il valore ritornato da `copy(...)` quando la chiami? E cosa contiene/rappresenta quel valore nel momento in cui viene creato (cioè appena prima di fare il primo .await su di esso)?
