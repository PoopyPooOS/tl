# Language Syntax

## Comments
This language uses the `//` prefix.  
As of right now there is no support for multi-line comments.

## Literals
Basic literals like numbers, strings, floats, booleans are the same as in every other language.  
<br>
Arrays are the same as other languages except they dont have comma seperators, like in Nix.  
<br>
Objects use an equals sign to seperate the key and value, there's also no commas seperating key-value pairs:
```tl
{
    key = value
}
```

## Variables
Variables can be defined with the `let` keyword:  
```tl
let x = 42
```

## String Interpolation
String interpolation can be done by using `${}` in any string:
```tl
let name = "John Doe"
let age = 42
let full_info = "My name is ${name} and I am ${age} year${if(age == 1 "" "s")} old."
```

## Functions
Functions are variables that have a value with the following syntax:
```tl
let greet = (name) {
    "Hello, my name is ${name}!"
}

greet("John Doe") // "Hello, my name is John Doe!"
```

## Logic & Branching
Branching is handled using functions:
```tl
let num = 10

if(
    // Condition
    num > 5
    // Then
    "Number is bigger than 5"
    // Else branch, another if statement (else if)
    if(
        num == 5
        "Number is exactly 5"
        "Number is less than 5"
    )
)
```
This requires a value to be explicitly defined on both branch sides.  
If the else branch doesn't return a value, `null` can be used instead but its recommended to have a proper value instead.

## Modules
Other files can be evaluated and imported with the `import` function:
```tl
// math.tl
{
    add = (a b) {
        a + b
    }

    sub = (a b) {
        a - b
    }
}
```
```tl
// main.tl
let math = import(./math.tl)

println(math.add(3 5)) // 8
println(math.sub(5 3)) // 2
```
