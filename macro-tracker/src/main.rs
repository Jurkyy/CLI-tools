use serde_json::{json, Value};
use std::error::Error;
use std::fs::File;
use std::io::Write;

async fn get_data_for_recipe(title: &str, ingredients: &[&str]) -> Result<Value, Box<dyn Error>> {
    let api_key = "1bd66313e38c52da7d301cba7d805cc3"; // Replace with your API key
    let app_id = "2a46a669"; // Replace with your app ID

    // Create a JSON payload with your recipe ingredients
    let payload = json!({
        "title": title,
        "ingr": ingredients
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

    match response.status() {
        reqwest::StatusCode::OK => {
            let response_text = response.text().await?; // Await the Result<String, reqwest::Error>
            let data: Value = serde_json::from_str(&response_text)?;
            Ok(data)
        }
        reqwest::StatusCode::NOT_MODIFIED => Err("Recipe not modified".into()),
        reqwest::StatusCode::NOT_FOUND => Err("Recipe not found".into()),
        reqwest::StatusCode::CONFLICT => Err("ETag token does not match the input data".into()),
        reqwest::StatusCode::UNPROCESSABLE_ENTITY => {
            Err("Unable to parse the recipe or extract nutritional info".into())
        }
        _ => Err(format!("HTTP Status {}: {}", response.status(), "Unknown error").into()),
    }
}

async fn write_data_to_file(title: &str, data: Value) -> Result<(), Box<dyn Error>> {
    // Create a file to write the data
    let mut file = File::create(format!("{}.txt", title))?;

    let mut total_protein = 0.0;
    let mut total_carbohydrates = 0.0;
    let mut total_fats = 0.0;

    // Iterate over the ingredients and their nutrients
    for (index, ingredient) in data["ingredients"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .enumerate()
    {
        if let Some(parsed) = &ingredient["parsed"].as_array() {
            let mut ingredient_protein = 0.0;
            let mut ingredient_carbohydrates = 0.0;
            let mut ingredient_fats = 0.0;

            for item in parsed.iter().flat_map(|v| v.as_object()) {
                if let Some(food) = &item["food"].as_str() {
                    // Write ingredient name with quantity and unit
                    if let Some(quantity) = item["quantity"].as_f64() {
                        if let Some(unit) = item["measure"].as_str() {
                            writeln!(file, "{}: {:.2} {}", food, quantity, unit)?;
                        } else {
                            writeln!(file, "{}: {:.2}", food, quantity)?;
                        }
                    } else {
                        writeln!(file, "{}", food)?;
                    }

                    // Write nutrient information for the ingredient
                    for (key, nutrient) in item["nutrients"]
                        .as_object()
                        .unwrap_or(&serde_json::Map::new())
                    {
                        if let Some(label) = nutrient["label"].as_str() {
                            if let Some(quantity) = nutrient["quantity"].as_f64() {
                                if let Some(unit) = nutrient["unit"].as_str() {
                                    writeln!(file, "{}: {:.2} {}", label, quantity, unit)?;

                                    // Accumulate nutrient totals for the ingredient
                                    match key.as_str() {
                                        "PROCNT" => ingredient_protein += quantity,
                                        "CHOCDF" => ingredient_carbohydrates += quantity,
                                        "FAT" => ingredient_fats += quantity,
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Add a line between ingredients
            if index < data["ingredients"].as_array().unwrap_or(&vec![]).len() - 1 {
                writeln!(file, "---")?;
            }

            // Accumulate nutrient totals for the overall total
            total_protein += ingredient_protein;
            total_carbohydrates += ingredient_carbohydrates;
            total_fats += ingredient_fats;
        }
    }

    // Write the total calories, protein, carbohydrates, and fats
    if let Some(total_calories_value) = data["totalNutrients"]["ENERC_KCAL"]["quantity"].as_f64() {
        writeln!(file, "\nTotal Calories: {:.2} kcal", total_calories_value)?;
        writeln!(file, "Total Protein: {:.2} g", total_protein)?;
        writeln!(file, "Total Carbohydrates: {:.2} g", total_carbohydrates)?;
        writeln!(file, "Total Fats: {:.2} g", total_fats)?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let title_recipe: &str = "my_recipe";
    let ingredients: Vec<&str> = vec![
        "6 bell peppers",
        "500g of rice",
        "1kg of chicken",
        "500ml of chicken broth",
        "2 large onions",
        "1kg of black beans",
        "200ml of olive oil",
    ]; // Replace with your recipe ingredients
    let data: Value = get_data_for_recipe(title_recipe, &ingredients).await?;
    write_data_to_file(title_recipe, data).await?;
    Ok(())
}
