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
let full_info = "My name is ${name} and I am ${age} year${if(age == 1, "", "s")} old."
```

## Functions
Functions are variables that have a value with the following syntax:
```
let greet = (name: str): str {
    "Hello, my name is ${name}!"
}

greet("John Doe") // "Hello, my name is John Doe!"
```

## Logic & Branching
Branching is handled using functions:
```
let num = 10

if(
    // Condition
    num > 5,
    // Then
    "Number is bigger than 5",
    // Else branch, another if statement (else if)
    if(
        num == 5,
        "Number is exactly 5",
        "Number is less than 5"
    )
)
```
This requires a value to be explicitly defined on both branch sides.  
If the else branch doesn't return a value, `null` can be used instead but its recommended to have a proper value instead.
> **Note:** The `null` value will be deprecated soon in favor of an enum similar to `Option` in Rust as it is more friendly to the type system.


## Note
If anything wasn't mentioned here, just assume it's like most other languages.