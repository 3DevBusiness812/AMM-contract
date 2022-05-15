pub fn arbitary_amounts(amount: f64, decimal: u8) -> f64 {
    let mut amount = amount;
    let mut decimal = decimal;
    let new_amount = loop {
        if decimal == 0 {
        break amount;
        }
        decimal -= 1;
        amount *= 10.0;
    };
    new_amount
}

pub fn normal_amount_fn(amount: f64, decimal: u8) -> f64 {
    let mut amount = amount;
    let mut _decimal = 0;
    let new_amount = loop {
        if _decimal == decimal {
        break amount;
        }
        _decimal += 1;
        amount /= 10.0;
    };
    new_amount
}
