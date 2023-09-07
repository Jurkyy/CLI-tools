use serde_json::{from_str, json, to_string, Value};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};

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

async fn write_data_to_file(
    title: &str,
    ingredients: &[&str],
    data: Value,
) -> Result<(), Box<dyn Error>> {
    // Create a file to write the data
    let mut file = File::create(format!("{}.txt", title))?;

    // Write the summary at the top of the output file
    writeln!(file, "#{}\n", title)?;

    // Calculate total nutrients
    if let Some(total_calories_value) = data["totalNutrients"]["ENERC_KCAL"]["quantity"].as_f64() {
        if let Some(total_protein_value) = data["totalNutrients"]["PROCNT"]["quantity"].as_f64() {
            if let Some(total_carbohydrates_value) =
                data["totalNutrients"]["CHOCDF"]["quantity"].as_f64()
            {
                if let Some(total_fats_value) = data["totalNutrients"]["FAT"]["quantity"].as_f64() {
                    writeln!(file, "Total Calories: {:.2} kcal", total_calories_value)?;
                    writeln!(file, "Total Protein: {:.2} g", total_protein_value)?;
                    writeln!(
                        file,
                        "Total Carbohydrates: {:.2} g",
                        total_carbohydrates_value
                    )?;
                    writeln!(file, "Total Fats: {:.2} g", total_fats_value)?;
                }
            }
        }
    }

    writeln!(file, "\n##Ingredients:\n")?;

    // Write the list of ingredients used as specified
    for ingredient in ingredients {
        writeln!(file, "{}", ingredient)?;
    }

    // Function to format labels
    fn format_label(label: &str) -> String {
        let mut formatted = String::new();
        let mut capitalize_next = true;

        for c in label.chars() {
            if c == '_' {
                formatted.push('-');
                capitalize_next = true;
            } else {
                formatted.push(if capitalize_next {
                    capitalize_next = false;
                    c.to_ascii_uppercase()
                } else {
                    c.to_ascii_lowercase()
                });
            }
        }

        formatted
    }

    // Write the diet labels
    writeln!(file, "\n##Diet Labels:\n")?;
    for diet_label in data["dietLabels"].as_array().unwrap_or(&vec![]) {
        if let Some(label) = diet_label.as_str() {
            writeln!(file, "- {}", format_label(label))?;
        }
    }

    // Write the health labels
    writeln!(file, "\n##Health Labels:\n")?;
    for health_label in data["healthLabels"].as_array().unwrap_or(&vec![]) {
        if let Some(label) = health_label.as_str() {
            writeln!(file, "- {}", format_label(label))?;
        }
    }

    // Add a line separator before the total nutrients
    writeln!(file, "\n##Total Nutrients:\n")?;

    // Iterate over the total nutrients and write them
    let total_nutrients = &data["totalNutrients"];
    for (_, nutrient) in total_nutrients
        .as_object()
        .unwrap_or(&serde_json::Map::new())
    {
        if let Some(label) = nutrient["label"].as_str() {
            if let Some(quantity) = nutrient["quantity"].as_f64() {
                if let Some(unit) = nutrient["unit"].as_str() {
                    writeln!(file, "{}: {:.2} {}", label, quantity, unit)?;
                }
            }
        }
    }

    // Add a line separator before the nutrient breakdown
    writeln!(file, "\n##Nutrient breakdown report:\n")?;

    // Iterate over the ingredients and their nutrients
    for (index, ingredient) in data["ingredients"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .enumerate()
    {
        if let Some(parsed) = &ingredient["parsed"].as_array() {
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
                                }
                            }
                        }
                    }
                }
            }

            // Add a line separator between ingredients
            if index < data["ingredients"].as_array().unwrap_or(&vec![]).len() - 1 {
                writeln!(file, "---")?;
            }
        }
    }

    // Add a line separator before the total nutrients
    writeln!(file, "---")?;

    Ok(())
}

fn write_value_to_file(data: &Value, filename: &str) -> Result<(), Box<dyn Error>> {
    let json_string = to_string(data)?; // Serialize to JSON string

    let mut file = File::create(filename)?;
    file.write_all(json_string.as_bytes())?;

    Ok(())
}

fn read_value_from_file(filename: &str) -> Result<Value, Box<dyn Error>> {
    let mut file = File::open(filename)?;
    let mut json_string = String::new();
    file.read_to_string(&mut json_string)?;

    let value = from_str(&json_string)?; // Deserialize from JSON string

    Ok(value)
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
       // let data: Value = get_data_for_recipe(title_recipe, &ingredients).await?;
       // write_value_to_file(&data, "temp_data.json");

    let data: Value = read_value_from_file("temp_data.json")?;
    write_data_to_file(title_recipe, &ingredients, data).await?;
    Ok(())
}
