BENCHMARK RESULTS - omnifetch

hyperfine, 10 runs, 3 warmup, static build. Lower is better.

Two ways of starting the program are measured:

VIA SHELL   the usual case: a terminal opens, your shell starts, and
            omnifetch is in its startup file, so the shell runs it for you.
NO SHELL    you type `omnifetch` yourself and it starts right away.

Modules = how many information blocks the command printed.


0. PROCESS STARTUP BASELINE

Empty static binary vs empty dynamic binary on the test machine:

Command                                Modules   Mean ms    Min ms
------------------------------------------------------------------
true (static, musl)                          0       0.57      0.2
/usr/bin/true (dynamic, glibc)               0       1.35      0.8


1. NO SHELL

Command                                Modules   Mean ms    Min ms
------------------------------------------------------------------
omnifetch -p minimal                         4       1.5       0.8
omnifetch --fast                            19       1.5       0.8
omnifetch                                   25       2.0       1.1
omnifetch -p compact                        13       2.0       1.1
omnifetch -p neofetch                       17       2.0       1.2
omnifetch -p fastfetch                      22       2.0       1.1
omnifetch -p hardware                       14       2.1       1.2
omnifetch -p modern                         19       2.2       1.3
omnifetch -p detailed                       33       2.5       1.5
nitch                                        9       3.0       2.0
paleofetch                                  11       3.3       2.1
omnifetch --no-cache                        25       3.7       2.3
pfetch                                       6       4.4       3.0
catnap                                      16       5.3       3.7
sysprint                                    23       7.6       5.7
omnifetch --all                             86       9.2       6.9
omnifetch --all --no-cache                  86      11.1       8.9
fastfetch                                   23      12.2       8.4
macchina                                    16      52.0      46.7
omnifetch --all --network --no-cache        88     192.6     106.9
omnifetch --network                         27     230.5     104.4
omnifetch --all --network                   88     238.1     103.5
omnifetch --network --no-cache              27     251.6     104.3
neofetch                                    16     636.6     628.1


2. VIA SHELL

Command                                Modules   Mean ms    Min ms
------------------------------------------------------------------
omnifetch -p minimal                         4       1.4       0.2
omnifetch --fast                            19       1.6       0.4
omnifetch -p neofetch                       17       1.9       0.5
omnifetch -p compact                        13       1.9       0.6
omnifetch                                   25       2.1       0.6
omnifetch -p hardware                       14       2.1       0.6
omnifetch -p fastfetch                      22       2.1       0.5
omnifetch -p modern                         19       2.2       0.8
omnifetch -p detailed                       33       2.5       0.9
nitch                                        9       2.9       1.2
paleofetch                                  11       3.3       1.7
omnifetch --no-cache                        25       3.9       2.2
pfetch                                       6       4.4       2.3
catnap                                      16       5.4       3.7
sysprint                                    23       7.7       5.8
omnifetch --all                             86       9.5       7.3
omnifetch --all --no-cache                  86      13.2       6.9
fastfetch                                   23      14.7       9.9
macchina                                    16      54.3      48.9
omnifetch --network --no-cache              27     231.1     106.6
omnifetch --all --network --no-cache        88     239.3     108.5
omnifetch --all --network                   88     247.7     106.9
omnifetch --network                         27     294.2     119.7
neofetch                                    16     678.8     649.7
