use shopify_function::prelude::*;
use shopify_function::Result;
use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize)]
struct Input {
    cart: Option<Cart>,
    discountNode: Option<DiscountNode>,
    presentmentCurrencyRate:Option<String>
}

#[derive(Deserialize, Serialize)]
struct Cart {
    lines: Option<Vec<CartLine>>
}
#[derive(Deserialize, Serialize)]
struct CartLine {
    quantity: i64,
    cost: CartLineCost,
    merchandise: Option<Merchandise>
   
}

#[derive(Deserialize, Serialize)]
struct ModifiedCartLine {
    quantity: i64,
    cost: f64,
    id: i64,
}
#[derive(Deserialize, Serialize)]
struct Merchandise {
    id: String,
    metafield: Option<Metafield>
}
#[derive(Deserialize, Serialize, PartialEq,Clone,Debug)]
struct Condition {
    quantity: i64,
    discount_value: f64,
    discount_type: String,
    discount_max_value: f64,
    discount_max_value_type: String,
}
#[derive(Deserialize, Serialize)]
struct CartLineCost {
    subtotalAmount: MoneyV2,
}

#[derive(Deserialize, Serialize)]
struct MoneyV2 {
    amount: String
}

#[derive(Deserialize, Serialize)]
struct DiscountNode {
    metafield: Option<Metafield>,
}

#[derive(Deserialize, Serialize)]
struct Metafield {
    value: Option<String>,
}

#[derive(Serialize)]
struct Output {
    discountApplicationStrategy: String,
    discounts: Vec<Discount>,
}

#[derive(Serialize)]
struct Discount {
    value: Value,
    targets: Vec<Target>,
    message: String,
}

#[derive(Serialize)]
struct Value {
    fixedAmount: FixedAmount,
}

#[derive(Serialize)]
struct FixedAmount {
    amount: f64,
}

#[derive(Serialize)]
struct Target {
    orderSubtotal: OrderSubtotal,
}

#[derive(Serialize)]
struct OrderSubtotal {
    excludedVariantIds: Vec<String>,
}

#[shopify_function]
fn function(input: Input) -> Result<Output> {
    let mut total_discount_amount : f64 = 0.0;
    let mut currencyRate =0.0;
    if let Some(present_currency_rate) = input.presentmentCurrencyRate{
        currencyRate = present_currency_rate.parse().unwrap();
        eprintln!("currency:{}", currencyRate); 
    } 
    // prepare cart_lines for easy usage
    // let mut cart_lines: Vec<ModifiedCartLine> = vec![];
    //let mut excludedVariantIds = vec![];
    if let Some(cart) = input.cart {
        if let Some(lines) = cart.lines {
            for line in lines {
                let mut line_item_discount_amount = 0.0;
                if let Some(merchandise) = &line.merchandise {
                    if let Some(metafield) = &merchandise.metafield {
                        if let Some(metafield_value) = metafield.value.as_ref().filter(|&value| value != "null") {
                       //  excludedVariantIds.push(merchandise.id.to_string());
                            // let json_string = serde_json::to_string_pretty(&metafield_value).unwrap();
                            // eprintln!("variants metafield_value: {}", json_string);  
                        let parsed_value: Vec<Condition> = serde_json::from_str(&metafield_value).unwrap();
                        let mut line_cost = line.cost.subtotalAmount.amount.parse::<f64>().unwrap_or(0.0);
                        line_cost=line_cost*currencyRate;
                        let quantity= line.quantity;
                       // println!("{:?}",parsed_value);
                        for con in parsed_value.iter() {
                         if con.quantity <= quantity {
                            //println!("Quantity match");
                            if con.discount_type == "F".to_string() {
                              //  println!("type match with F");
                            // eprintln!("discount_value{}", con.discount_value);    
                                line_item_discount_amount = con.discount_value * currencyRate * quantity as f64;
                                if con.discount_max_value_type == "P".to_string() {
                                    let Price_of_single_variant = (line_cost) / quantity as f64;
                                    let max_discount_value = con.discount_max_value * Price_of_single_variant  * 0.01;
                                    if con.discount_max_value > 0.0 && line_item_discount_amount > max_discount_value  {
                                    line_item_discount_amount =  max_discount_value;
                                    }
                                }
                                else
                                {
                                    if con.discount_max_value > 0.0 && line_item_discount_amount > con.discount_max_value  {
                                        line_item_discount_amount = con.discount_max_value * currencyRate;
                                    }  
                                }

                            }
                            else if con.discount_type == "P".to_string() {
                              //  println!("type match with P");
                                line_item_discount_amount = con.discount_value * line_cost  * 0.01;
                                if con.discount_max_value_type == "P".to_string() {
                                    let Price_of_single_variant = (line_cost) / quantity as f64;
                                    let max_discount_value = con.discount_max_value * Price_of_single_variant  * 0.01;
                                    if con.discount_max_value > 0.0 && line_item_discount_amount > max_discount_value  {
                                    line_item_discount_amount =  max_discount_value;
                                    }
                                }
                                else
                                {
                                    if con.discount_max_value > 0.0 && line_item_discount_amount > con.discount_max_value  {
                                        line_item_discount_amount = con.discount_max_value * currencyRate;
                                    }  
                                }
                             
                            }
                            else
                            {
                                let Price_of_single_variant = (line_cost) / quantity as f64;
                                let Single_variant_discount= Price_of_single_variant - (con.discount_value * currencyRate);
                                line_item_discount_amount = Single_variant_discount * quantity as f64;
                                if con.discount_max_value_type == "P".to_string() {
                                    let max_discount_value = con.discount_max_value * Price_of_single_variant  * 0.01;
                                    if con.discount_max_value > 0.0 && line_item_discount_amount > max_discount_value  {
                                    line_item_discount_amount =  max_discount_value;
                                    }
                                }
                                else
                                {
                                    if con.discount_max_value > 0.0 && line_item_discount_amount > con.discount_max_value  {
                                        line_item_discount_amount = con.discount_max_value * currencyRate;
                                    }  
                                }

                            }
                         }
                         }

                        total_discount_amount = total_discount_amount + line_item_discount_amount;
                
                  }
                  }
                }
            }   
        }
    }
   // let joined = excludedVariantIds.join(", ");
   // eprintln!("excludedVariantIds: {}", joined);
    //total_discount_amount=total_discount_amount*currencyR;
   
    if total_discount_amount < 0.1 {
        eprintln!("No cart lines qualify for discount.");
        let output = Output {
            discountApplicationStrategy: String::from("FIRST"),
            discounts: vec![],
        };
        Ok(output)
    }
    else
    {
        let targets = vec![Target {
            orderSubtotal: OrderSubtotal {
                excludedVariantIds: vec![],
            },
        }];
        let message = String::from("Bulk Discount");
        let output = Output {
            discountApplicationStrategy: String::from("FIRST"),
            discounts: vec![Discount {
                value: Value {
                    fixedAmount: FixedAmount {
                        amount: total_discount_amount,
                    },
                },
                targets: targets,
                message: message,
            }],
        };
        Ok(output)
    }
}

#[cfg(test)]
mod tests;
