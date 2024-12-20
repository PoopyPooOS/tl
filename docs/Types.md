# Language Types

## Type Inference
When defining variables the type of the variable is inferred, you can not explicitly state the type in a `let` statement.

## Generics
Generics are pretty similar to other languages except you can also have a dynamic amount of generics, this is mainly used for the function arguments in the `fn` type.

## Built-In Types
Primitives: `bool`, `num`, `float`, `str`  
Other types: `array<T>`, `object<K, V>`  
Function type: `fn<T...>`

## User Defined Types
```
struct Person {
    name = str
    age = num

    new = (name: str, age: num): Person {
        Person { name age }
    }

    is_adult = (self): bool {
        self.age >= 18
    }
}

// The type `array<Person>` is inferred here
let family = [
    Person.new("John Doe", 43)
    Person.new("Jane Doe", 41)
    Person.new("Baby Doe", 2)
]

print(if(
    family.all(Person.is_adult),
    "All the family members are adults",
    if(
        family.any(Person.is_adult),
        "Not all the family members are adults",
        "None of the family members are adults"
    )
))
```
Types are checked before evaluation so user defined types use a seperate keyword from `let`.