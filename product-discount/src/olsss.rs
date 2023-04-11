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
#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct Configuration {
    pub discount: i64,
    pub variants: f64,
    pub variant: String,
}
impl Configuration {
    const DEFAULT_QUANTITY: i64 = 999;
    const DEFAULT_PERCENTAGE: f64 = 0.0;
    const DEFAULT_VARIANT: &'static str = "";
    // Parse the JSON metafield value using serde
    fn from_str(value: &str) -> Self {
        serde_json::from_str(value).expect("Unable to parse configuration value from metafield")
    }
}
impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            quantity: Self::DEFAULT_QUANTITY,
            percentage: Self::DEFAULT_PERCENTAGE,
            variant:Self:: DEFAULT_VARIANT.to_string()
        }
    }
}

#[shopify_function]
fn function(input: input::ResponseData) -> Result<output::FunctionResult> {
    let no_discount = output::FunctionResult {
        discounts: vec![],
        discount_application_strategy: output::DiscountApplicationStrategy::FIRST,
    };

    // Specify the product variant IDs that should receive the discount
    //let variant_ids = vec!["gid://shopify/ProductVariant/43623862337751"];
    
    
    let config = match input.discount_node.metafield {
        Some(input::InputDiscountNodeMetafield { value }) =>
            Configuration::from_str(&value),
        None => return Ok(no_discount),
    };
    let variant = None;

    let target = input.cart.lines
        .iter()
        // Only include cart lines with a targetable product variant
        .filter_map(|line| match &line.merchandise {
            input::InputCartLinesMerchandise::ProductVariant(v) => {
                variant = Some(v);
                variant
            },
            input::InputCartLinesMerchandise::CustomProduct => None,
        })
    .collect::<Vec<_>>();
    let SaleLog=0;
        for item in config.iter() {
           let data=item.variants;
           if(data.contains(variant.id))
           {
           SaleLog=item;
           }        
        }
    //terate all the lines in the cart to create discount targets
    let targets = input.cart.lines
        .iter()
        // Only include cart lines with a quantity higher than two
        .filter(|line| line.quantity >= config.quantity)
        // Only include cart lines with a targetable product variant
        .filter_map(|line| match &line.merchandise {
            input::InputCartLinesMerchandise::ProductVariant(variant) => Some(variant),
            input::InputCartLinesMerchandise::CustomProduct => None,
        })
        // Only include variants with matching IDs
        .filter(|variant| variant.id.as_str() == config.variant.as_str())
        // Use the variant ID to create a discount target
        .map(|variant| output::Target {
            product_variant: Some(output::ProductVariantTarget {
                id: variant.id.to_string(),
                quantity: None,
            })
        }).collect::<Vec<output::Target>>();

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
                    value: config.percentage.to_string()
                })
            }
        }],
        discount_application_strategy: output::DiscountApplicationStrategy::FIRST,
    })
}

#[cfg(test)]
mod tests;
