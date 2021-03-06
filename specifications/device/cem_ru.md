CELL MEMORY (или сокращенно CEM)
---
CEM - устройство памяти (несовсем) последовательного доступа имеющее следующий вид.
также у устройства должен быть хотя бы один порт.

Основная суть - возможность псевдо-"вставки" ячеек памяти между ячейками **MM** (см. ниже) и их последующее удаление.

###### скорее всего проще понять на примерах, они расположены снизу

### внутренняя структура:
+ основная память **MM** (main memory)
+ регистр текущей позиции MM 
+ дополнительная память **AM** (additional memory)
+ регистр текущей позиции AM. регистр последней чистой позиции AM. триггер перехода AM.
+ память хранящая порядок **OM** (order memory)
+ регистр {последней | текущей} позиции OM-AM. регистр {последней | текущей} позиции MM-AM.
+ тригер текущей памяти CUR-AM 
+ регистр текущего количетсва в посл. OM (принимает значения от 1 до 7) 

#### MM, AM, OM
каждая из данных памятей по отдельности является обычной памятью последовательного доступа, с размером ячейки = байту.

#### триггер перехода AM
находится в состоянии 0 если значение регистра текущей позиции AM меньше значения регистр последней чистой позиции AM иначе в состоянии 1.  

#### ячейки памяти OM
каждый байт представлен следующим образом  
```
|     1 bit      |        3 bit        |     1 bit      |        3 bit        | 
|overflow-mm-flag|consecutive-mm-amount|overflow-am-flag|consecutive-am-amount|
```
если длина подряд идущих ячеек памяти одного типа больше `0b111` то соотв. ему `overflow-flag` ставится в `T` 


### устройство должно уметь:
+ переходить к следующему байту
+ переходить к предыдущему байту
+ передавать текущий байт в порт
+ устанавливать текущий байт в полученное из порта значение
+ "создавать" AM клетку если последняя чистая позиция AM меньше или равна текущей
+ "удалять" AM клетку если последняя чистая позиция AM меньше или равна текущей  

#### создание AM клетки
обновляет OM регистры соотв. образом (см. примеры).  
увеличивает регистр текущей позиции AM на 1

#### удаление AM клетки
можно удалять только порожденные клетки и только в случае если текущая позиция AM больше или равна последней чистой позиции AM.

обновляет OM регистры соотв. образом (см. примеры).  
если текущая позиция AM равна последней чистой позиции то обнуляет текущую AM клетку и уменьшаем на 1 регистр последней чистой позиции AM. 
уменьшает регистр текущей позиции AM на 1

### примеры 
введем обозначения
+ **>** следующая клетка 
+ **<** предыдущая клетка 
+ **\[X\]** установка значения тек. клетки в X 
+ **c** создание клетки
+ **d** удаление клетки 
+ **↓** текущая позиция 
+ **С** клетка является порожденной
+ **X** на этой или влюбом месте после этой клетке клетку можно создать, а если при этом клетка является C то ее можно удалить 
для удобства все числа записаны в hex форме

#### последовательный пример 1:
выполним:  
`>[42]c[10]>[20]`  
получим:  
```
OM: |F2F1|F1F0|F0F0|...
MM: |00|42|20|00|...
AM: |10|00|...
+ значения регистров
```
если это представить в линейной форме то получится:
```
           ↓
|00|42|10|20|00|..
        C
        X
```

после выполним:  
`>>>[15]>>>[25]>>>[35]`  
получим:  
```
OM: |F2F1|T7F0|F1F0|F0F0|...
MM: |00|42|20|00|00|15|00|25|00|35|00|...
AM: |10|00|...

                                ↓
|00|42|10|20|00|00|15|00|25|00|35|00|...
        C
        X
```

после выполним:  
`<<<cc[11]c<cc`  
получим:  
```
OM: |F2F1|F5F5|F3F0|F0F0|...
MM: |00|42|20|00|00|15|00|25|00|35|00|...
AM: |10|00|11|00|00|00|00|...

                                   ↓
|00|42|10|20|00|00|15|00|00|11|00|00|00|25|00|35|00|...
        C                 C  C  C  C  C
                             X
```

после выполним:  
`ddd`  
получим:  
```
OM: |F2F1|F5F2|F3F0|F0F0|...
MM: |00|42|20|00|00|15|00|25|00|35|00|...
AM: |10|00|00|00|...

                          ↓
|00|42|10|20|00|00|15|00|00|00|25|00|35|00|...
        C                 C  C
                          X
```
после выполним:  
`dd`  
получим:  
```
OM: |F2F1|T7F0|F1F0|F0F0|...
MM: |00|42|20|00|00|15|00|25|00|35|00|...
AM: |10|00|...

                       ↓
|00|42|10|20|00|00|15|00|25|00|35|00|...
        C              
        X
```


#### пример 2:
если выполнить 30 раз `>` то OM будет следующей:

```
OM: |T7F0|T7F0|T7F0|T7F0|F2F0|F0F0|...
```






















