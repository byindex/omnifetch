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
omnifetch -p minimal                         4       3.3       1.6
omnifetch -p fastfetch                      22       3.7       2.0
omnifetch -p hardware                       14       3.9       2.0
omnifetch -p detailed                       33       4.4       2.4
omnifetch --fast                            19       4.5       1.8
nitch                                        9       4.9       2.6
paleofetch                                  11       5.1       2.9
omnifetch -p neofetch                       17       5.8       2.0
omnifetch                                   14       6.5       2.5
omnifetch -p compact                        13       7.2       2.2
pfetch                                       6       7.3       4.0
omnifetch --no-cache                        14       7.4       2.6
omnifetch -p modern                         19       8.4       2.5
catnap                                      16      10.0       5.1
sysprint                                    23      17.6       8.0
omnifetch --all --no-cache                  82      19.4      10.8
fastfetch                                   23      21.8      12.1
omnifetch --all                             82      27.3      12.3
macchina                                    16      70.2      52.7
omnifetch --network --no-cache              16     149.8     112.0
omnifetch --all --network                   84     158.9     110.5
omnifetch --network                         16     163.7     112.8
omnifetch --all --network --no-cache        84     208.0     123.3
neofetch                                    16    1485.4    1135.0


2. VIA SHELL

Command                                Modules   Mean ms    Min ms
------------------------------------------------------------------
nitch                                        9       2.2       0.0
paleofetch                                  11       2.8       0.0
omnifetch -p minimal                         4       3.6       0.0
pfetch                                       6       3.8       1.2
omnifetch -p compact                        13       4.5       0.0
omnifetch -p neofetch                       17       4.6       0.0
omnifetch --fast                            19       4.9       0.0
omnifetch --no-cache                        14       5.0       0.1
omnifetch -p modern                         19       6.1       0.0
omnifetch -p fastfetch                      22       6.4       0.4
catnap                                      16       6.8       2.0
sysprint                                    23       7.7       4.1
omnifetch -p detailed                       33       8.0       1.5
omnifetch                                   14       9.2       0.0
omnifetch -p hardware                       14       9.8       0.1
fastfetch                                   23      14.7       9.1
omnifetch --all                             82      14.9       6.5
omnifetch --all --no-cache                  82      18.1       8.7
macchina                                    16      63.9      52.6
omnifetch --all --network --no-cache        84     148.3     114.1
omnifetch --network                         16     169.6     109.2
omnifetch --all --network                   84     186.3     115.9
omnifetch --network --no-cache              16     193.1     112.2
neofetch                                    16    1282.5     775.6

