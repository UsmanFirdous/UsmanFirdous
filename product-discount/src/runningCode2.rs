use shopify_function::prelude::*;
use shopify_function::Result;
use serde::{Deserialize,Serialize};
generate_types!(
    query_path = "./input.graphql",
    schema_path = "./schema.graphql"
);

#[derive(Debug, Clone, Serialize)]
struct ProductVariant {
    id: String,
}
// #[derive(Serialize, Deserialize, PartialEq,Default,Debug)]
// #[serde(rename_all(deserialize = "camelCase"))]
// struct Configuration {
//     logs: Vec<Nodes>,
// }
#[derive(Serialize, Deserialize, PartialEq,Clone,Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
struct Nodes {
    discount: Vec<Discount>,
    variants: Vec<String>,
}

#[derive(Serialize, Deserialize, PartialEq,Clone,Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
struct Discount {
    value: f32,
    quantity: u32,
    #[serde(rename = "type")]
    discount_type: String,
}
#[shopify_function]
fn function(input: input::ResponseData) -> Result<output::FunctionResult> {
    let no_discount = output::FunctionResult {
        discounts: vec![],
        discount_application_strategy: output::DiscountApplicationStrategy::FIRST,
    };
    let config: Vec<Nodes> = match input.discount_node.metafield {
        Some(input::InputDiscountNodeMetafield { value }) => { 
       // println!("metafield_value: {:?}", value);
        serde_json::from_str(&value).unwrap()
       },
        None => return Ok(no_discount),
    };
   // println!("config{:?}",config);
   // let mut flag=0;
    let mut dis = Vec::new();
    for node in config.iter() {
            let mut targets = Vec::new();  
            let mut d_quantity:u32 =0;
            for line in input.cart.lines.iter() {
                if let input::InputCartLinesMerchandise::ProductVariant(variant) = &line.merchandise {
                        if node.variants.contains(&variant.id) {
                          //  println!("variant matched.");
                            let mut d_type="null";
                            let mut d_value:f32=0.0;
                            for discount in node.discount.iter() {
                               // println!("discount lop{:?}",discount); 
                                if discount.quantity <= line.quantity as u32
                                {
                                d_quantity=discount.quantity;
                                d_type=discount.discount_type.as_str();
                                d_value=discount.value;
                                }
                            }
                            // println!("quantity {}",d_quantity); 
                            // println!("discount type {}",d_type);
                            // println!("cart quantity {}",line.quantity);
                            if line.quantity >= d_quantity as i64 {
                             //   println!("quantity match successffully.");    
                            targets.push(output::Target {
                                product_variant: Some(output::ProductVariantTarget {
                                    id: variant.id.to_string(),
                                    quantity: None,
                                })
                            });
                            let discount_value =  if d_type=="P"
                            {
                               // println!("type matched successfully.");
                               output::Value {
                                fixed_amount: None,
                                percentage: Some(output::Percentage {
                                    value: d_value.to_string(),
                                }),
                               }
                                    
                            }
                            else
                            {
                                output::Value {
                                    fixed_amount: Some(output::FixedAmount {
                                        amount: d_value.to_string(),
                                        applies_to_each_item: Some(true),
                                    }),
                                    percentage: None,
                                }
                            };
                            dis.push(output::Discount {
                                message: None,
                                targets: targets.clone(),
                                value: discount_value,
                            });
                          }
                        }
                    }
                }


        }
        if dis.is_empty() {
            eprintln!("No cart lines qualify for volume discount.");
            return Ok(no_discount);
        }
    
        Ok(output::FunctionResult {
            discounts:dis,
            discount_application_strategy: output::DiscountApplicationStrategy::FIRST,
        })   
   
}
#[cfg(test)]
mod tests;



