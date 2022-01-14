### opcodes
| comand name name | standart instruction | byte sequence | description |
|--|--|--|--|
| PASS | not exist | 0x00 |do nothing|
| SET | **s** | **COM**: 0x01 **CEM**: SE | TODO:STOP:HERE |
| INC | **+** | 0xXX | increases curent mem cell value by one |
| DEC | **-** | 0xXX | decreases curent mem cell value by one |
| NEXTC | **>** | 0xXX | moves the mem cell pointer to the next |
| PREVC | **<** | 0xXX | moves the mem cell pointer to the previous one |

###### mem is short for memory

### additional designations
#### in byte sequence section
+ binary operation A:B means that after A (any element from set A) comes B (any element from B)
+ S : Small byte, it has a value from 0 to 127 [0x00; 0x7F]  
+ B : Big byte, it has a value from 128 to 255 [0x80; 0xFF]
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
