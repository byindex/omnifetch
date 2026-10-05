ПРЕСЕТЫ ВЁРСТКИ - omnifetch
============================

Пресет — это фиксированный набор модулей в фиксированном порядке. Восемь из
них повторяют дефолтный вывод конкурента один в один, так что скорость можно
сравнивать на одинаковых данных. В колонке «Модулей» столько информационных
модулей, сколько пресет выполняет на самом деле, без служебных элементов
вокруг них.

  omnifetch --list-presets           все пресеты с описаниями

  omnifetch -p fastfetch             ровно то, что показывает fastfetch
  omnifetch -p neofetch              ...и neofetch
  omnifetch -p sysprint
  omnifetch -p catnap
  omnifetch -p macchina
  omnifetch -p paleofetch
  omnifetch -p nitch
  omnifetch -p pfetch

  omnifetch -p detailed              34 модуля
  omnifetch -p all                   86 модулей, дефолтный набор плюс добавки


Пресет       Модулей  Что внутри
---------   -------  -----------------------------------------------
minimal           5  только OS, kernel, uptime, память
pfetch            7  как pfetch
nitch             8  как nitch
paleofetch       12  как paleofetch
compact          14  плотно, без пустых строк
hardware         15  аудит железа
catnap           15  как catnap
macchina         16  как macchina
sysprint         17  как sysprint, по секциям
neofetch         18  как neofetch 7.x
modern           20  температуры, нагрузка, сеть
fastfetch        23  как fastfetch
detailed         34  максимум информации без мониторинга
all              86  все локальные модули, --network добавляет два остальных