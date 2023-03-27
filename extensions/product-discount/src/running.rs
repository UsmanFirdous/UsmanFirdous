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
    // Specify the product variant IDs that should receive the discount
    //let variant_ids = vec!["gid://shopify/ProductVariant/43623862337751"];
    let config: Vec<Nodes> = match input.discount_node.metafield {
        Some(input::InputDiscountNodeMetafield { value }) => { 
       // println!("metafield_value: {:?}", value);
        serde_json::from_str(&value).unwrap()
       },
        None => return Ok(no_discount),
    };
    // let discount=Vec<discount>
     
    let cartVariants= input.cart.lines
    .iter()
    .filter(|line| line.quantity >= 5)
    .filter_map(|line| match &line.merchandise {
            input::InputCartLinesMerchandise::ProductVariant(variant) => Some(variant),
            input::InputCartLinesMerchandise::CustomProduct => None,
    }).collect::<Vec<_>>();
    
    for line in &input.cart.lines {
        let line_quantity = line.quantity;
        println!("Quantity: {}", line_quantity);
    }

    println!("Variants: {:?}", cartVariants);
    // let alltargets = input.cart.lines;
    // println!("alltargets: {:?}", alltargets);
    let targets = input.cart.lines
    .iter()
    .filter(|line| line.quantity >= 5)
    .filter_map(|line| match &line.merchandise {
            input::InputCartLinesMerchandise::ProductVariant(variant) => Some(variant),
            input::InputCartLinesMerchandise::CustomProduct => None,
    })
    .filter(|variant| config.iter().any(|node| node.variants.contains(&variant.id)))
    .map(|variant| output::Target {
        product_variant: Some(output::ProductVariantTarget {
            id: variant.id.to_string(),
            quantity: None,
        })
    })
    .collect::<Vec<output::Target>>();
    println!("target: {:?}", targets);
    if targets.is_empty() {
        eprintln!("No cart lines qualify for volume discount.");
        return Ok(no_discount);
    }
    Ok(output::FunctionResult {
        discounts: vec![output::Discount {
            message: None,
            targets,
            // Define a percentage-based discount
            value: output::Value {
                fixed_amount: None,
                percentage: Some(output::Percentage {
                    value: 10.to_string()
                })
            }
        }],
        discount_application_strategy: output::DiscountApplicationStrategy::FIRST,
    })
}

#[cfg(test)]
mod tests;
