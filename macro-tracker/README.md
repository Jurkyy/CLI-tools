# Recipe Nutrient Analyzer

This is a Rust application that analyzes the nutritional content of recipes based on ingredients. It takes a JSON input containing information about ingredients and their nutritional values, calculates the total nutrient content for each ingredient, and provides a summary of the total nutrients for the entire recipe.

## Features

- Parses JSON input containing ingredient nutritional information.
- Calculates and displays the total nutrient content for each ingredient.
- Generates a summary of the total nutrients for the entire recipe, including calories, protein, carbohydrates, and fats.

## Usage

1. Clone the repository:

   ```bash
   git clone <repository-url>
   ```

2. Build the project:

   ```bash
   cargo build
   ```

3. Run the application:
   ```bash
   cargo run <path-input-file>
   ```

4. View the analyzed recipe:
   The output of the tool should write to a .txt file with the name of your recipe's title.
   Example of the layout can be seen in the my_recipe.txt file.

## License

This project is licensed under the MIT License - see the LICENSE.md file for details.