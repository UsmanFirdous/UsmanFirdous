use shopify_function::prelude::*;
use shopify_function::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Input {
    cart: Option<Cart>,
    discountNode: Option<DiscountNode>,
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
    id: i64
}

#[derive(Deserialize, Serialize)]
struct Merchandise {
    id: String,
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
    // prepare cart_lines for easy usage
    let mut cart_lines: Vec<ModifiedCartLine> = vec![];
    if let Some(cart) = input.cart {
        if let Some(lines) = cart.lines {
            for line in lines {
                if let Some(merchandise) = &line.merchandise {
                    let line_cost = line.cost.subtotalAmount.amount.parse::<f64>().unwrap_or(0.0);
                    let id_parts: Vec<&str> = merchandise.id.split("/").collect();
                    let merchandise_id = id_parts.last().unwrap().parse()?;

                    let cart_line = ModifiedCartLine {
                        cost: line_cost,
                        quantity: line.quantity,
                        id: merchandise_id,
                    };
                    cart_lines.push(cart_line);
                }
            }   
        }
    }

    if let Some(discount_node) = input.discountNode {
        if let Some(metafield) = discount_node.metafield {
            if let Some(metafield_value) = metafield.value {
                let json_string = serde_json::to_string_pretty(&metafield_value).unwrap();
                eprintln!("metafield_value: {}", json_string);
                let parsed_value: serde_json::Value = serde_json::from_str(&metafield_value)?;
                // Access the rules field of the Value object
                if let Some(rules) = parsed_value.get("rules") {
                    eprintln!("rules: {}", rules);
                    if rules.is_array() {
                        for rule in rules.as_array().unwrap() {
                            let mut line_item_discount_amount = 0.0;
                            if let Some(condition) = rule.get("condition") {
                                if let Some(condition_str) = condition.as_str() {
                                    eprintln!("condition: {}", condition_str);
                                    if let Some(rule_qty) = rule.get("qty") {
                                        let rule_variant_ids = match rule.get("variant_ids") {
                                            Some(variant_ids) => {
                                                let variant_ids_vec = variant_ids.as_array().unwrap();
                                                variant_ids_vec.iter().map(|id| id.as_i64().unwrap()).collect()
                                            },
                                            None => vec![],
                                        };
                                        for line in &cart_lines {
                                            if rule_variant_ids.contains(&line.id) {
                                                eprintln!("Variant id matched: {}", line.id);
                                                let mut is_discount_applicable = false;
                                                if condition_str == "gte" {
                                                    if line.quantity >= rule_qty.as_i64().unwrap() {
                                                        eprintln!("Variant id greater than: {}", rule_qty);
                                                        is_discount_applicable = true;
                                                    }
                                                } else if condition_str == "eq" {
                                                    if line.quantity == rule_qty.as_i64().unwrap() {
                                                        is_discount_applicable = true;
                                                    }
                                                } else if condition_str == "lte" {
                                                    if line.quantity <= rule_qty.as_i64().unwrap() {
                                                        is_discount_applicable = true;
                                                    }
                                                }
                                                if is_discount_applicable {
                                                    if let Some(discount) = rule.get("discount") {
                                                        if let Some(amount) = discount.get("amount") {
                                                            if let Some(discount_type) = discount.get("type") {
                                                                if let Some(discount_type_str) = discount_type.as_str() {
                                                                    if discount_type_str == "percent" {
                                                                        line_item_discount_amount = amount.as_f64().unwrap() * line.cost * 0.01;
                                                                    } else if discount_type_str == "flat" {
                                                                        line_item_discount_amount = amount.as_f64().unwrap();
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            total_discount_amount = total_discount_amount + line_item_discount_amount;
                        }
                    }
                } else {
                    eprintln!("Rules not found");
                }
            }
        }
    }    

    let mut targets = vec![Target {
        orderSubtotal: OrderSubtotal {
            excludedVariantIds: vec![],
        },
    }];
    let mut message = String::from("Bevy Discount Test");
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

#[cfg(test)]
mod tests;
