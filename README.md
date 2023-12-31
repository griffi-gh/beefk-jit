<h1>beefk</h1>
Fast, optimized JIT compiler for <a href="https://esolangs.org/wiki/Brainfuck">Brainfuck</a> written in Rust.

<image src=".assets/m.png" height="300">

<table><tr><td>
  <div>
    <b>Warning: generated native code <i>intentionally</i> lacks array bound checks!</b><br>
    This compiler is optimized for <i>performance</i> and does not care about memory safety
  </div>
</td><tr></table>

<h1>Benchmarks</h1>

CPU: Ryzen 5 5625U with 16GB of dual-channel DDR4 RAM running at 3200 MHz\
Only <i>execution</i> time is counted, stdout is piped to `/dev/null` during the benchmark

<table>
  <tr>
    <th></th>
    <th><b><code>beefk</code></b></th>
    <th><a href="https://github.com/griffi-gh/brian"><code>griffi-gh/brian</code></a></th>
  </tr>
  <tr>
    <th><a href=".bf/mandelbrot.bf"><code>mandelbrot.bf</code></a></th>
    <td>~600ms</td>
    <td>~3400ms</td>
  <tr>
</table>
