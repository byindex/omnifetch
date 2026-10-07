# Пресеты вёрстки - omnifetch

Пресет — это фиксированный набор модулей в фиксированном порядке. Восемь из них повторяют дефолтный вывод конкурента один в один, так что скорость можно сравнивать на одинаковых данных. В колонке «Модулей» указано количество информационных модулей, которое пресет выполняет на самом деле, без служебных элементов оформления.

```bash
omnifetch --list-presets           # все пресеты с описаниями

omnifetch -p fastfetch             # ровно то, что показывает fastfetch
omnifetch -p neofetch              # ...и neofetch
omnifetch -p sysprint
omnifetch -p catnap
omnifetch -p macchina
omnifetch -p paleofetch
omnifetch -p nitch
omnifetch -p pfetch

omnifetch -p detailed              # 34 модуля
omnifetch -p all                   # 86 модулей, дефолтный набор плюс добавки
```

## Доступные пресеты

| Пресет | Модулей | Что внутри |
| :--- | :---: | :--- |
| `minimal` | 5 | Только OS, kernel, uptime, память |
| `pfetch` | 7 | Как pfetch |
| `nitch` | 8 | Как nitch |
| `paleofetch` | 12 | Как paleofetch |
| `compact` | 14 | Плотно, без пустых строк |
| `hardware` | 15 | Аудит железа |
| `catnap` | 15 | Как catnap |
| `macchina` | 16 | Как macchina |
| `sysprint` | 17 | Как sysprint, по секциям |
| `neofetch` | 18 | Как neofetch 7.x |
| `modern` | 20 | Температуры, нагрузка, сеть |
| `fastfetch` | 23 | Как fastfetch |
| `detailed` | 34 | Максимум информации без мониторинга |
| `all` | 86 | Все локальные модули (`--network` добавляет два остальных) |