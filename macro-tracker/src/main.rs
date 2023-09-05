use reqwest;
use serde::Deserialize;
use serde_json::{json, Value};
use std::error::Error;

#[derive(Deserialize)]
struct IngredientInfo {
    calories: f64,
    // Other nutritional information fields
}

async fn get_calories_for_recipe(ingredients: &[&str]) -> Result<f64, Box<dyn Error>> {
    let api_key = "1bd66313e38c52da7d301cba7d805cc3"; // Replace with your API key
    let app_id = "2a46a669"; // Replace with your app ID

    // Create a JSON payload with your recipe ingredients
    let ingredients_list: Vec<&str> = ingredients.iter().map(|&ingredient| ingredient).collect();
    let payload = json!({
        "title": "My Recipe",
        "ingr": ingredients_list
    });

    let url = format!(
        "https://api.edamam.com/api/nutrition-details?app_id={}&app_key={}",
        app_id, api_key
    );

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(payload.to_string())
        .send()
        .await?;

    let response_text = response.text().await?; // Await the Result<String, reqwest::Error>
    let data: Value = serde_json::from_str(&response_text)?;

    // Extract the total calories from the response
    if let Some(calories_info) = data["totalNutrients"]["ENERC_KCAL"]["quantity"].as_f64() {
        return Ok(calories_info);
    }

    Err("Calories not found in the response")?
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let ingredients = vec!["1 Apple", "1 Banana", "1 onion"]; // Replace with your recipe ingredients
    let total_calories = get_calories_for_recipe(&ingredients).await?;
    println!("Total Calories in Recipe: {:.2}", total_calories);
    Ok(())
}
