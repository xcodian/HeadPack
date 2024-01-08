# The HeadPack Object Format

**TL;DR** It's like JSON, and it's comparable to MessagePack with some improvements.
```
       JSON: {"easy":true,"as":{"pi":3.1415927}}            (35 bytes)

MessagePack: 82a4 easy c3a2 as 81a2 pi cb400921fb5a7ed197   (23 bytes)
   
   HeadPack: 8f21c089a44492 easyaspi 40490fdb               (19 bytes)
             ~~~~~~~~~~~~~~ ~~~~~~~~~~~~~~~~~
          detachable header              data
```
## Terminology

### Message
A JSON document encoded using HeadPack is called a *Message*. A *Message* can
contain multiple root *Object*s, and is effectively analogous to a JSON document
with the root element of `[]`.

^ NOT ANYMORE!!!

### Object
Everything that carries data or gives structure to a HeadPack message is an
*Object*. If you convert this JSON message to HeadPack, you would have `10`
total objects. Notice how both collections like `Map` & `List` and data types
like `String`, `SInt`, `Bool` and `Float` count as objects.

```jsonc
// this is the root element, doesn't count
{
    // String (1)  String (2)
    "hello":       "world",
    // String (3)
    "stuff": [  // List (4)
        1,      // SInt (5)
        2,      // SInt (6)
        3,      // SInt (7)
        true,   // Bool (8)
        3.14159 // Float (9)
    ]
}
```

Objects are flattened in a Message in the order they appear. For example,
the above message would be serialized as
```
type: Map  String   String   String   List  SInt SInt SInt  Bool  Float
data:      "hello"  "world"  "stuff"        1    2    3     true  3.14159
``` 

### Classes
In HeadPack, there are `12` total Object "types" which can compose a structure.
These are intended to be a superset of the types commonly seen in JSON, such as `String`, `Bool`, etc.

However, reserving `4` bits just to store a type id would be inefficient, and so
HeadPack has a more efficient system for type information storage.

In HeadPack, all types fall under four categories calls type *Classes*. Because
there are only four of them, they can be identified using just two bits each.

#### List of Type Classes

- `ID 0` **String**
  - Human-readable variable-length text encoded in UTF-8.
- `ID 1` **Bytes**
  - Variable length contiguous array of raw bytes.
- `ID 2` **Collection**
    - List
      - Array of objects. Equivalent to JSON's `[ a, b, c, ... ]`.
    - Map
      - Key-value dictionary. Equivalent to JSON's `{ a: b }`.
      - Yes, map keys can be *any* Object, even maps.
- `ID 3` **Numeric**
    - `SInt` - Signed Integer
      - Always occupies the least number of space required to store.
      - Between `8` and `128` bits in size.
    - `UInt` - Unsigned Integer 
      - Always occupies the least number of space required to store.
      - Between `8` and `128` bits in size.
    - `Float32` - Single-precision decimal
      - Equivalent to C's `float` or Rust's `f32`
    - `Float64` - Double-precision decimal
      - Equivalent to C's `double` or Rust's `f64`
    - `Null`
      - Note that this is its own type. Objects are not arbitrarily nullable.
    - `Timestamp32`
      - Second-level precision Unix timestamp
      - Equivalent to a fixed `32-bit` unsigned integer
    - `UserDefined`
      - Fixed length that **MUST** be specified by user.
      - Contains an `ID` in the range `39` to `63`.

### Lengths
In order to read the data properly, the decoder needs to know how long the data
stored in an Object is. You can imagine this is especially important with
`String`'s and `Bytes`.

Furthermore, as you can see, save for `String` and `Bytes`, it's not enough to
just know an Object's Class to be able to figure out its type. 
- An object of class Numeric can be 7+ different types!
- Is this Collection a `List` or a `Map`?

In the case of collections and Numeric classes, types are determined by
encoding information in the length value of the object.

#### Collections
For collections, if the lowest bit of length is `1`, then the type is `List`,
otherwise it's `0`. The actual length (in elements) of the collection is the
rest of the bits taken from the second lowest.
- Example 1, given the length `00000100`:
  - Check the last bit, `0000010 -> 0 <-`
    - It's a `0`, so this must be a `Map`!
  - The length of our map is `0000010` = `2` key-value pairs
  - Result: This is a `Map` with length `2`
- Example 2, given the length `00011101`:
  - Check the last bit, `0001110 -> 1 <-`
    - It's a `1`, so this must be a `List`!
  - The length of our list is `0001110` = `14` list elements
  - Result: This is a `List` with length `14`

#### Numeric
Let `L` be the length of an object `O` of class Numeric.

- if `1 ≤ L ≤ 16`, then `O` is an `SInt` with length `L`
- if `17 ≤ L ≤ 32`, then `O` is a `UInt` with length `L - 16`
- if `L = 33`, then `O` is a `Float32` with length `4`
- if `L = 34`, then `O` is a `Float64` with length `8`
- if `L = 35`, then `O` is a `Null` with length `0`
- if `L = 36`, then `O` is a `Bool` set to `false` with length `0`
- if `L = 37`, then `O` is a `Bool` set to `true` with length `0`
- if `L = 38`, then `O` is a `Timestamp32` with length `4`
- if `39 ≤ L ≤ 63`, then `O` is a `UserDefined` with ID `L` and user-specified length

**Congratulations!** Now that we know how to parse and Object from its 2-bit class and
its length, we are now ready to delve into how HeadPack formats Messages.

## Message Format
HeadPack relies on three sections: the *`CLASS`* section, the *`LENGTH`* section and the *`DATA`* section.

#### Legend
### `CLASS` Section
This is the first section in a Message, and it contains type information
regarding the class of each object. It's effectively an array of Classes
represented by their 2-bit ID. Because each ID is two bits, we can fit up to
three class definitions into a single byte, and reserve the last two bits as the
count (0 to 3) of classes in the next byte.

For example, suppose we had 5 objects (A, B, C, D, E) for which we needed to
store the classes. We could put the classes of A, B, C in the first 6 bits of a
byte, followed by the number 2 (`10` in binary) to signal that the next byte
contains only D & E's classes, and extrapolate this pattern, for larger numbers
of classes.

The first two bits of the first byte must then be reserved to signal how many
classes are stored in the first byte. The last two bits of the first byte must also be reserved to signal how many classes are stored in the second byte, and then the normal pattern described above continues.

```
Legend: 
  . means that bit is irrelevant
  bits 0 to 7 are shown as 01 23 45 67

====== Example 1 ======

first byte (chunk):
      10 AA BB 10 <-- finally, a 2 again because the next chunk contains 2 more classes
      ^  ^^ ^^ 
      |   \__|____________________________________________________________
      |                                                                   \
      two bits storing "2" because both A & B's class id's are stored in here

second byte:
      DD EE .. ..  <-- the 2 at the end of the last chunk said that 
                       only D & E's 2-bit ids are contained here

====== Example 2 ======

first byte:
      10 AA BB 11  <-- 11 in binary = 3
      ^^
      2 again, so A & B are in the first byte like before

second byte:
      CC DD EE 11  <-- another 3 at the end, so F, G & H must be in the next chunk
      ^^ ^^ ^^
      because of the 3 at the end of the first chunk, we see C, D & E here

      FF GG HH 01  <-- 01 in binary = 1, so the next chunk is the last one
      II .. .. ..      and it only contains 1 more class id

====== Example 3 ======

first byte:
      11 AA BB CC
      ^^
      a 3 here means that there are only 3 total 
      objects to store, so this is the only chunk!

====== Example 4 ======

first byte:
      01 AA .. ..
      ^^
      a 1 here means that there is only 1 total 
      object to store
```
And that's pretty much it about the class section! If you decoded it properly,
you should have an array of object classes which:

1. Tells you how many objects there are in total.
2. Tells you a little bit about the structural hierarchy of the message, though
   you don't know if collections are maps or lists yet.

### `LENGTH` section

### `DATA` section

