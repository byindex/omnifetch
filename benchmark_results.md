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
omnifetch -p minimal                         4       2.0       1.0
omnifetch --fast                            19       2.0       1.0
omnifetch -p fastfetch                      22       2.2       1.4
omnifetch                                   25       2.2       1.3
omnifetch -p neofetch                       17       2.3       1.4
omnifetch -p detailed                       33       2.5       1.8
omnifetch -p compact                        13       2.6       1.4
omnifetch -p hardware                       14       2.7       1.4
omnifetch -p modern                         19       3.0       1.5
nitch                                        9       4.2       2.4
omnifetch --no-cache                        25       4.7       2.7
pfetch                                       6       5.2       3.7
paleofetch                                  11       5.7       2.6
catnap                                      16       6.6       4.5
sysprint                                    23       8.8       6.1
omnifetch --all --no-cache                  82      12.4       9.2
omnifetch --all                             82      13.2       7.9
fastfetch                                   23      16.5      11.2
macchina                                    16      61.2      49.3
omnifetch --network --no-cache              27     166.6     127.7
omnifetch --network                         27     190.2     117.7
omnifetch --all --network --no-cache        84     200.0     125.0
omnifetch --all --network                   84     278.6     121.2
neofetch                                    16     790.1     700.8


2. VIA SHELL

Command                                Modules   Mean ms    Min ms
------------------------------------------------------------------
omnifetch -p minimal                         4       2.0       0.4
omnifetch -p compact                        13       2.6       0.8
omnifetch -p modern                         19       3.0       1.0
nitch                                        9       3.1       1.6
paleofetch                                  11       3.2       1.7
omnifetch                                   25       3.3       0.9
omnifetch --fast                            19       3.4       0.5
omnifetch -p hardware                       14       5.2       0.9
pfetch                                       6       5.5       2.8
catnap                                      16       5.7       3.6
omnifetch -p neofetch                       17       6.2       0.8
omnifetch -p fastfetch                      22       7.4       1.0
sysprint                                    23       8.7       5.4
omnifetch --no-cache                        25       8.8       2.2
omnifetch -p detailed                       33      12.0       1.6
omnifetch --all                             82      14.5       8.2
fastfetch                                   23      15.2      10.2
omnifetch --all --no-cache                  82      19.1       8.5
macchina                                    16      60.3      47.3
omnifetch --all --network                   84     351.6     141.2
omnifetch --network                         27     358.1     129.5
omnifetch --all --network --no-cache        84     365.6     113.3
omnifetch --network --no-cache              27     463.1     133.6
neofetch                                    16     714.0     640.2

