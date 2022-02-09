# Реализованные эмуляторы устройств
+ [std-cem](https://github.com/Nikita-str/brainfuck-ext--cell-gen-and-devs/blob/master/src/bfcg/dev_emulators/cem/std_cem.rs)  
устройство эмулирующее CEM "стандартным" образом.  
под стандартным в данном случае понимается что у каждой из операций ниже есть определенный код, после которого, если нужно, передается/считывается значение.
  + создать клетку
  + удалить клетку
  + перейти к следующей клетке
  + перейти к предыдущей клетке
  + получить значение текущей клетки 
  + присвоить текущей клетке значение    
**дополнительные параметры**:  
  + mm-sz - размер MM памяти в байтах
  + am-sz - размер AM памяти в байтах
+ [std-com](https://github.com/Nikita-str/brainfuck-ext--cell-gen-and-devs/blob/master/src/bfcg/dev_emulators/com/com.rs)  
устройство эмулирующее COM - память которая каждый раз возвращает следующую клетку, а также умеет перемещаться вперед и назад на переданное значение (в SE форме).  
дополнительные параметры:  
  + mem-sz - размер памяти в байтах
+ [num-console](https://github.com/Nikita-str/brainfuck-ext--cell-gen-and-devs/blob/master/src/bfcg/dev_emulators/console/console_num.rs)  
устройство эмулирующее консоль считывающая числа от 0 до 255 и выводящая числа от 0 до 255.  
дополнительные параметры:  
  + new-line - после скольких выведенных чисел нужно переводить строку (требуется небольшое исправление) 
+ [ascii-console](https://github.com/Nikita-str/brainfuck-ext--cell-gen-and-devs/blob/master/src/bfcg/dev_emulators/console/console_ascii.rs)  
устройство эмулирующее консоль считывающая и выводящая ascii символы (32-126 + пробельные символы).  
дополнительные параметры: нету
+ [utf8-console](https://github.com/Nikita-str/brainfuck-ext--cell-gen-and-devs/blob/master/src/bfcg/dev_emulators/console/console_utf8.rs)  
устройство эмулирующее консоль считывающая и выводящая utf8 символы.   
дополнительные параметры: нету
+ [std-win](https://github.com/Nikita-str/brainfuck-ext--cell-gen-and-devs/blob/master/src/bfcg/dev_emulators/win/dev_win.rs)  
устройство эмулирующее экран (в данный момент может быть подключено только единственное подобное устройство)  
дополнительные параметры:
  + w - ширина
  + h - высота
  + x+y - начальная позиция окна экрана 


