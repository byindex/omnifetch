BENCHMARK RESULTS - omnifetch

hyperfine, 10 runs, 3 warmup, static build. Lower is better.

Two ways of starting the program are measured:

VIA SHELL   the usual case: a terminal opens, your shell starts, and
            omnifetch is in its startup file, so the shell runs it for you.
NO SHELL    you type `omnifetch` yourself and it starts right away.

Modules = how many information blocks the command printed.


1. NO SHELL

Command                                Modules   Mean ms    Min ms
------------------------------------------------------------------
omnifetch -p minimal                         4       1.4       0.9
omnifetch --fast                            19       1.5       1.1
omnifetch -p compact                        13       1.7       1.3
omnifetch -p neofetch                       17       1.7       1.3
omnifetch -p hardware                       14       1.8       1.4
omnifetch                                   25       1.9       1.4
omnifetch -p fastfetch                      22       1.9       1.4
omnifetch -p modern                         19       1.9       1.5
omnifetch -p detailed                       33       2.3       1.7
paleofetch                                  11       2.8       2.4
nitch                                        9       3.0       2.2
omnifetch --no-cache                        25       3.4       2.2
pfetch                                       6       4.5       3.5
catnap                                      16       5.3       4.4
omnifetch --all                             82       5.4       4.2
omnifetch --all --no-cache                  82       7.1       5.9
sysprint                                    23       7.8       6.2
fastfetch                                   23      16.3      10.7
macchina                                    16      70.0      47.9
omnifetch --all --network --no-cache        84     114.0     107.1
omnifetch --all --network                   84     114.6     106.3
omnifetch --network                         27     139.2     103.1
omnifetch --network --no-cache              27     155.7     103.2
neofetch                                    16     586.6     569.9


2. VIA SHELL

Command                                Modules   Mean ms    Min ms
------------------------------------------------------------------
omnifetch                                   25       1.6       1.5
omnifetch -p hardware                       14       1.8       1.4
omnifetch -p compact                        13       1.8       1.4
omnifetch -p fastfetch                      22       1.8       1.4
omnifetch -p minimal                         4       1.8       1.0
omnifetch -p modern                         19       1.8       1.5
omnifetch --fast                            19       1.9       1.1
omnifetch -p neofetch                       17       1.9       1.4
omnifetch -p detailed                       33       2.0       1.7
omnifetch --no-cache                        25       3.0       2.6
nitch                                        9       3.4       2.3
paleofetch                                  11       3.4       2.4
pfetch                                       6       4.1       3.4
omnifetch --all                             82       4.7       4.3
catnap                                      16       5.9       4.3
omnifetch --all --no-cache                  82       6.7       6.0
sysprint                                    23       7.3       6.1
fastfetch                                   23      13.6      10.7
macchina                                    16      49.8      46.6
omnifetch --all --network                   84     111.4     102.8
omnifetch --network                         27     120.0     102.4
omnifetch --network --no-cache              27     132.9     105.2
omnifetch --all --network --no-cache        84     192.6     104.4
neofetch                                    16     634.3     595.8

