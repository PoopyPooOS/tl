let pow = (base: int, exponent: int): int {
    if(
        exponent == 0,
        1,
        base * pow(base, exponent - 1)
    )
}

pow(2, 10)