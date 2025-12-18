// A comment

{
  bool = [
    true
    false
  ]
  int = 42
  float = 3.14
  string = "Hello, world!"
  escaped_string = "Hello, \n\tworld!"
  interpolated_string = "Hello, my name is ${name}!"
  relative_path = [ ./file.txt ../file.txt ]
  absolute_path = /file.txt
  object = { name = "John Doe" age = 42 }
  member_access = package.dependencies
  array = [ 1 2 3 ]
  not = !true
  function_declaration = [
    name: "Hello, ${name}!"
    name: age: "Hello, ${name}! You are ${age} years old."
  ]
  call = [
    dbg("Hello, world!", null)
    if(
      password == "strongpassword123"
      "Password is correct"
      "Password is incorrect"
    )
  ]
  binary_op = 2 + 3 * 4
  bindings = let
    name = "John Doe"
  in
    null
}
