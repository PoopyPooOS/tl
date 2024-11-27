# Language Syntax

## Comments
This language uses the `//` prefix.  
As of right now there is no support for multi-line comments.

## Literals
Basic literals like numbers, strings, floats, booleans are the same as in every other language.  
<br>
Arrays are the same as other languages except they dont have comma seperators, like in Nix.  
<br>
Objects can use both an equals or a colon to seperate the key and value, there's also no commas seperating key-value pairs:
```
{
    key = value
    // or
    key: value
}
```

## Variables
Variables can be defined with the `let` keyword:  
```
let x = 42
```

## String Interpolation
String interpolation can be done by using `${}` in any string:
```
let name = "John Doe"
let age = 42
let full_info = "My name is ${name} and I am ${age} years old."
```

## Functions
Functions can be defined with the `fn` keyword, and called with generic syntax:
```
fn greet(name) {
    println("Hello, my name is ${name}")
}

greet("John Doe")
```

## Note
If anything wasn't mentioned here, just assume it's like most other languages.