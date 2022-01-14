### additional designations
#### in byte sequence section
+ binary operation A:B means that after A (any element from set A) comes B (any element from B)
+ S : Small byte it has a value from 0 to 127 [0x00; 0x7F]  
+ B : Big byte it has a value from 128 to 255 [0x80; 0xFF]
+ SE : Set of byte sequences which have the following form: S, B:S, B:..:B:S  
#### in description section
+ std se decoding: means that the value from SE is decoded as follows:  
```
sh = 0;
result = 0;
loop 
  {
  byte = get_next_byte();
  result += (byte & max(S)) << sh;
  if byte > max(S) { sh += 7; }
  else { break; } 
  }
```

-------------------

### minimal processor opcodes : all operations by devices
| comand name name | standart instruction | byte sequence | description |
|--|--|--|--|
| PASS | not exist | 0x00 |do nothing|
| SET | **s** | 0x01:SE | TODO:STOP:HERE |

хм задумался: можем ли мы не использовать для операций с устройствами cell memory? 
наверное можем но нам придется добавить операцию SETS (set by stream) чтобы принимать SE через поток байт, хотя у нас есть test port для этих целей... хм интересно
и так:  
SET: set cur port on REG  
WR: write in cur port ... second reg?! => не нравится, но видимо можно swap'ами регистров в который читаем это сделать  
хех, наверное забавно, можно будет и сделать  
SWAP: swap cur reg  
Z: zero cur reg  
MUL: mul on 128 (<<7) cur reg  
TEST: test port
READ: read from port and reg = reg & 0xF..F80 + READED_VALUE & 0x7F ? 
не, как то очень странно выходит, но вообще этого должно хватить чтобы написать все то что было запланировано  
ладно, на сегодня устал, но думаю не с этого явно стоит начинать  
=> rename to bad_way__opcodes.md


### additional opcodes for direct cell manipulations 
| comand name name | standart instruction | byte sequence | description |
|--|--|--|--|
| INC | + | 0xXX | increases curent cell value by one |
| DEC | - | 0xXX | decreases curent cell value by one |
| NEXTC | - | 0xXX | moves the cell pointer to the next |
| PREVC | - | 0xXX | moves the cell pointer to the previous one |
