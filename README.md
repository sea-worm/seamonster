```sh
cargo run -- <csv-file>
```

## Completeness

All five transaction kinds have been implemented.

My understanding of the dispute process was that a dispute can only be opened for a deposit transaction.
The other option would have been that withdrawal transactions can also be disputed.

## Correctness

>  How do you know this? Did you test against sample data?

I added a few test cases in [`src/bank/test.rs`](src/bank/test.rs) while I was in the TDD loop.
I also generated a few dummy CSV transaction CSV files for "integration" testing.

> If so, include it in the repo. 

It has been explicitly requested _not_ to include sample data.

> Did you write unit tests for the complicated bits?

Some tricky edge cases (e.g. double chargeback), but not all edge cases are covered due to limited time.

> Or are you using the type system to ensure correctness?

Yes, extensive use of pattern matching and error types. This doesn't guarantee the absence of errors (especially logic errors) though.

## Safety and Robustness

> Are you doing something dangerous?

Using in-memory data structures (as data would be lost on a crash) and a single-threaded process (as we couldn't handle multiple transactions in parallel).

> Tell us why you chose to do it this way. How are you handling errors?

Very basic error handling via the built-in `Result` type has been added.
Unparsable CSV rows trigger a hard panic, but logically inapplicable transactions are softly skipped.
The motivation for panicking on invalid CSV rows during development is simple: catch errors quickly.
In production, I would opt to skip invalid CSV rows and only trigger alerts (or better I would move them into a dead-letter queue).

## Efficiency

> Be thoughtful about how you use system resources. Sample data sets may
be small but you may be evaluated against much larger data sets (hint:
transaction IDs are valid u32 values).

I have done no performance optimizations (limited time).
With more time, I would look into a better data structure and reduce the memory footprint by storing only the fields and data that are absolutely relevant.

Of course, in practice, a sole in-memory data structure would obviously not be a good idea (single point of failure, no backups).

As a small optimization I have decided to store the transactions for each account in a separate map, s.t.
- lookups and frees can be faster (though probably tweaking the general-purpose HashMap might help more here)
- (with more time) one could have channels or actors per account and process transactions in parallel 

> Can you stream values through memory as opposed to loading the entire data set upfront?

This is already the case. I'm using a buffered CSV iterator.

> What if your code was bundled in a server, and these CSVs came from thousands of
concurrent TCP streams

I would make sure that an account can be aggregated independently, s.t. each server (and/or it's respective thread) can handle a small subset (shard) of accounts.
A common solution would be to use a streaming-processing platform like e.g. Kafka, Flink or Spark.