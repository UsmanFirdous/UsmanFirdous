use shopify_function::prelude::*;
use shopify_function::Result;
use serde::{Serialize};

generate_types!(
    query_path = "./input.graphql",
    schema_path = "./schema.graphql"
);

#[derive(Debug, Clone, Serialize)]
struct ProductVariant {
    id: String,
}

#[shopify_function]
fn function(input: input::ResponseData) -> Result<output::FunctionResult> {
    let no_discount = output::FunctionResult {
        discounts: vec![],
        discount_application_strategy: output::DiscountApplicationStrategy::FIRST,
    };

    // Specify the product variant IDs that should receive the discount
    let variant_ids = vec!["gid://shopify/ProductVariant/43623862337751"];

    // Iterate all the lines in the cart to create discount targets
    let targets = input.cart.lines
        .iter()
        // Only include cart lines with a quantity higher than two
        .filter(|line| line.quantity >= 2)
        // Only include cart lines with a targetable product variant
        .filter_map(|line| match &line.merchandise {
            input::InputCartLinesMerchandise::ProductVariant(variant) => Some(variant),
            input::InputCartLinesMerchandise::CustomProduct => None,
        })
        // Only include variants with matching IDs
        .filter(|variant| variant_ids.contains(&variant.id.as_str()))
        // Use the variant ID to create a discount target
        .map(|variant| output::Target {
            product_variant: Some(output::ProductVariantTarget {
                id: variant.id.to_string(),
                quantity: None,
            })
        })
        .collect::<Vec<output::Target>>();

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
                    value: "10.0".to_string()
                })
            }
        }],
        discount_application_strategy: output::DiscountApplicationStrategy::FIRST,
    })
}

#[cfg(test)]
mod tests;
