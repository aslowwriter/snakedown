### Benchmarking

Benchmarking/profilling to improve performance is tricky, because so many smart and complicated tricks are already done for you by compilers/operating systems/hardware etc. meaning that you have to make sure you work together with them instead of against them. Performance measuring and improving is part science, part art. To accommodate for this we have designed our benchmarks with the following considerations:

- Results of the benchmarks are intentionally not saved because there is an inherent amount of irreproducibility to them
- While the results are not saved, the _setup_ is. All benchmarks should use exactly the config used in `benchmark-config.toml` for the snakedown configuration and the `bench` profile for compilation (unless you are benchmarking different profiles). If you modify any of these you'll have to run new baselines!
- Remember that benchmarks are inherently hardware bound, so do not pay too much attention to the individual numbers, and if you are bench marking yourself, remember to always run the baselines yourself.
- There are currently two benchmark commands in the `pixi.toml` we use:
    - `bench` this runs the citereon benchmark. Use this if you want to see if your changes had an impact on performance
    - `flamegraph` Runs the benchmark config under `cargo flamegraph` which will produce a graph that can help you gain insight into which parts of the code need optimising. Knowing how to read this graph can be tricky at first. See the [flamegrpah repo](https://github.com/flamegraph-rs/flamegraph) for more information on this
