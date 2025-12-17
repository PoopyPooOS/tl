with {
  greet = |name: string|: list<foobar> "Hello, ${name}!"
}
greet("world")
