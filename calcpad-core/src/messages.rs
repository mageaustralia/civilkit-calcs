//! English message strings, generated from Calcpad.Core/Messages.resx by tools/gen_messages.py.
#![allow(non_upper_case_globals, dead_code)]

pub const Invalid_syntax_0: &str = "Invalid syntax: \"{0}\".";
pub const Incomplete_expression: &str = "Incomplete expression.";
pub const Missing_left_bracket: &str = "Missing left bracket \"(\".";
pub const Missing_right_bracket: &str = "Missing right bracket \")\".";
pub const Invalid_symbol_0: &str = "Invalid symbol: \"{0}\".";
pub const Invalid_units_0: &str = "Invalid units: \"{0}\".";
pub const Cannot_evaluate_0_as_1: &str = "Cannot evaluate \"{0}\" as {1}.";
pub const Invalid_operator_0: &str = "Invalid operator: \"{0}\".";
pub const Power_must_be_unitless: &str = "Power must be unitless.";
pub const The_expression_on_the_right_does_not_evaluate_to_units: &str =
    "The expression on the right does not evaluate to units.";
pub const Cannot_evaluate_the_function_f_for_v_equals_0: &str = "Cannot evaluate the function %F for %V = {0}.";
pub const The_function_f_is_not_defined_for_v_equals_0: &str = "The function %F is not defined for %V = {0}.";
pub const Inconsistent_units_0_and_1: &str = "Inconsistent units: \"{0}\" and \"{1}\".";
pub const Limits_out_of_range_0_1: &str = "Limits out of range: [{0}; {1}].";
pub const Invalid_units_for_function_0_1: &str = "Invalid units for function: \"{0}({1})\".";
pub const Root_index_must_be_unitless: &str = "Root index must be unitless.";
pub const Root_index_cannot_be_a_complex_number: &str = "Root index cannot be a complex number.";
pub const Root_index_must_be_integer_more_than_1: &str = "Root index must be an integer > 1.";
pub const Argument_out_of_range_for_n_factorial: &str = "Argument out of range for n! .";
pub const The_argument_of_n_factorial_must_be_unitless: &str = "The argument of n! must be unitless.";
pub const The_argument_of_n_factorial_must_be_a_positive_integer: &str =
    "The argument of n! must be a positive integer.";
pub const The_argument_of_n_factorial_cannot_be_complex: &str = "The argument of n! cannot be complex.";
pub const Cannot_evaluate_remainder_0_1_Denominator_must_be_unitless: &str =
    "Cannot evaluate remainder: \"{0}  %  {1}\". Denominator must be unitless.";
pub const Both_values_must_be_integers: &str = "Both values must be integers.";
pub const Variable_0_does_not_exist: &str = "Variable '{0}' does not exist.";
pub const Unit_0_does_not_exist: &str = "Unit '{0}' does not exist.";
pub const Interrupted_by_user: &str = "Interrupted by user.";
pub const Calculations_not_active: &str = "Calculations not active.";
pub const Expression_is_empty: &str = "Expression is empty.";
pub const Missing_operand: &str = "Missing operand.";
pub const Invalid_function_0: &str = "Invalid function: \"{0}\".";
pub const Stack_memory_leak_Invalid_expression: &str = "Stack memory leak. Invalid expression.";
pub const Stack_empty_Invalid_expression: &str = "Stack empty. Invalid expression.";
pub const Undefined_input_field: &str = "Undefined input field.";
pub const Undefined_variable_or_units_0: &str = "Undefined variable or units: \"{0}\".";
pub const Error_evaluating_0_as_function: &str = "Error evaluating \"{0}\" as function.";
pub const Error_evaluating_0_as_function_or_operator: &str = "Error evaluating \"{0}\" as function or operator.";
pub const Cannot_rewrite_existing_units_0: &str = "Cannot rewrite existing units: {0}.";
pub const The_calculated_units_0_are_inconsistent_with_the_target_units_1: &str =
    "The calculated units \"{0}\" are inconsistent with the target units \"{1}\".";
pub const Invalid_character_exception: &str = "Invalid character: '{0}'. Variables, functions and units must begin with a letter or ∡, except for the following units: ° ′ ″ % ‰.";
pub const Improper_use_of_the_assignment_operator_equals: &str = "Improper use of the assignment operator '='.";
pub const Missing_left_bracket_in_solver_command: &str = "Missing left bracket '{' in solver command.";
pub const Missing_right_bracket_in_solver_command: &str = "Missing right bracket '}' in solver command.";
pub const Invalid_macro_identifier_0: &str = "Invalid macro identifier: \"{0}$\".";
pub const Invalid_solver_command_definition_0: &str = "Invalid solver command definition \"{0}\".";
pub const Error_parsing_0_as_units: &str = "Error parsing \"{0}\" as units.";
pub const Error_parsing_0_as_number: &str = "Error parsing \"{0}\" as number.";
pub const Missing_delimiter_0_in_solver_command_1: &str = "Missing item or delimiter \"{0}\" in solver command: {1}.";
pub const More_than_one_operator_equals_in_0: &str = "More than one operators '=' in '{0}'.";
pub const The_expression_on_the_right_side_must_be_constant_0: &str =
    "The expression on the right side must be constant: \"{0}\".";
pub const Inconsistent_units_for_0_equals_1: &str = "Inconsistent units for \"{0} = {1}\".";
pub const Inconsistent_units_for_0_equals_1_2: &str = "Inconsistent units for \"{0} = {1} : {2}\".";
pub const No_solution_for_0: &str = "No solution for: {0}.";
pub const Recursion_is_not_allowed_in_function_definition_0: &str =
    "Recursion is not allowed in function definition: \"{0}\".";
pub const The_assignment_equals_must_be_preceded_by_custom_function_or_variable: &str =
    "The assignment '=' must be preceded by custom function or variable.";
pub const Assignment_equals_must_be_the_first_operator_in_the_expression: &str =
    "Assignment '=' must be the first operator in the expression.";
pub const Invalid_syntax_0_1: &str = "Invalid syntax: \"{0} {1}\".";
pub const Unexpected_delimiter: &str = "Unexpected delimiter ';'.";
pub const Invalid_number_of_arguments: &str = "Invalid number of arguments.";
pub const The_result_is_not_a_real_number_0: &str = "The result is not a real number: \"{0}\".";
pub const Missing_parameter_in_function_definition: &str = "Missing parameter in function definition.";
pub const Missing_delimiter_in_function_definition: &str = "Missing delimiter in function definition.";
pub const Invalid_token_in_function_definition_0: &str = "Invalid token in function definition: \"{0}\".";
pub const Circular_reference_detected_for_function_0: &str = "Circular reference detected for function \"{0}\".";
pub const Circular_reference_detected_for_macro_0: &str = "Circular reference detected for macro \"{0}\".";
pub const Circular_include_detected_0: &str = "Circular #include detected: \"{0}\".";
pub const Invalid_function_definition_exception: &str =
    "Invalid function definition. It has to match the pattern: \"f(x; y; z...) = expression\".";
pub const Argument_out_of_range_for_0_x: &str = "Argument out of range for {0}(x).";
pub const Condition_cannot_be_empty: &str = "Condition cannot be empty.";
pub const Condition_block_not_initialized_with_if: &str = "Condition block not initialized with \"#if\".";
pub const Duplicate_else_in_condition_block: &str = "Duplicate \"#else\" in condition block.";
pub const else_if_is_not_allowed_after_else_in_condition_block: &str =
    "\"#else if\" is not allowed after \"#else\" in condition block.";
pub const Condition_cannot_evaluate_to_a_complex_number: &str = "Condition cannot evaluate to a complex number.";
pub const Condition_result_is_invalid_0: &str = "Condition result is invalid: {0}.";
pub const Duplicate_argument_0: &str = "Duplicate argument for parameter \"{0}\".";
pub const Duplicate_macro_parameter_names_0_and_1: &str = "Duplicate macro parameter names: {0} and {1}.";
pub const Macro_not_defined_0: &str = "Macro not defined: {0}.";
pub const Required_parameter_after_optional_0: &str = "Required parameter \"{0}\" cannot follow an optional parameter.";
pub const Unknown_keyword_argument_0: &str = "Unknown keyword argument: \"{0}\".";
pub const Missing_0_in_surface_map_command: &str = "Missing {0} in surface map command.";
pub const The_limits_of_plot_area_are_identical: &str = "The limits of plot area are identical.";
pub const Error_writing_a_png_file_to_0: &str = "Error writing a png file to \"{0}\".";
pub const Error_writing_a_svg_file_to_0: &str = "Error writing a svg file to \"{0}\".";
pub const Error_converting_png_to_Base64: &str = "Error converting png to Base64.";
pub const Inconsistent_units_0_1_2: &str = "Inconsistent units: \"{0} {1} {2}\".";
pub const Units_cannon_be_raised_to_complex_power: &str = "Units cannon be raised to complex power.";
pub const Cannot_evaluate_remainder_0_1_The_denominator_must_be_unitless: &str =
    "Cannot evaluate remainder: \"{0}  ⦼  {1}\". The denominator must be unitless.";
pub const if_block_not_closed_Missing_end_if: &str = "Error: \"#if\" block not closed. Missing \"#end if\".";
pub const Iteration_block_not_closed_Missing_loop: &str = "Error: Iteration block not closed. Missing \"#loop\".";
pub const Unexpected_error_0_Please_check_the_expression_consistency: &str =
    "Unexpected error: {0} Please check the expression consistency.";
pub const Paused_Press_F5_to_continue: &str = "<p><span class=\"err\">Paused!</span> Press <b>F5</b> to <a href=\"#0\" data-text=\"continue\">continue</a> or <b>Esc</b> to <a href=\"#0\" data-text=\"cancel\">cancel</a>.</p>";
pub const Error_found_on_line: &str = "<div class=\"errorHeader\">Found <b>1</b> error on line:";
pub const Errors_found_on_lines: &str = "<div class=\"errorHeader\">Found <b>{0}</b> errors on lines:";
pub const Error_in_0_on_line_1_2: &str = "Error in \"{0}\" on line {1}: {2}";
pub const Error_in_0_1: &str = "Error in \"{0}\": {1}";
pub const Number_of_iterations_exceeds_the_maximum_0: &str = "Number of iterations exceeds the maximum {0}.</p>";
pub const loop_without_a_corresponding_repeat: &str =
    "\"#loop\" without a corresponding \"#repeat\", \"#for\" or \"#while\".";
pub const Entangled_if__end_if__and_repeat__loop_blocks: &str =
    "Entangled \"#if - #end if\" and \"#repeat - #loop\" blocks.";
pub const continue_without_a_corresponding_repeat: &str = "\"#continue\" without a corresponding \"#repeat\".";
pub const Macro_definition_block_not_closed_Missing_end_def: &str =
    "Error: Macro definition block not closed. Missing \"#end def\".";
pub const Missing_source_file_for_include: &str = "Missing source file for include.";
pub const File_not_found: &str = "File not found.";
pub const Invalid_macro_name_0: &str = "Invalid macro name: \"{0}\".";
pub const Invalid_variable_name_0: &str = "Invalid variable name: \"{0}\".";
pub const Invalid_inside_macro_definition: &str = "Invalid inside macro definition.";
pub const There_is_no_matching_def: &str = "\"There is no matching \"#def\".";
pub const Invalid_symbol_0_in_macro_name: &str = "Invalid symbol \"{0}\" in macro name.";
pub const Duplicate_macro_name_0: &str = "Duplicate macro name: \"{0}\".";
pub const Missing_delimiter_0_in_plot_command: &str =
    "<span class = \"err\">Missing delimiter \"{0}\" in plot command \"{1}\".</span>";
pub const Missing_0_in_plot_command_1: &str = "<span class = \"err\">Missing {0} in plot command \"{1}\".</span>.";
pub const More_than_one_delimiter_in_plot_command: &str =
    "<span class = \"err\">More than one delimiter \"|\" in plot command {0}.</span>";
pub const Missing_delimiter_0_in_surface_map_command_1: &str =
    "<span class = \"err\">Missing delimiter \"{0}\" in surface map command \"{1}\".</span>";
pub const Function_redefined: &str = " Warning! Function redefined.";
pub const Raise_to_square: &str = "Raise to square";
pub const Too_many_errors: &str = "Too many errors. Further calculations are skipped.";
pub const repeat_block_not_closed_Missing_loop: &str = "Repeat block not closed. Missing \"#loop#.";
pub const The_output_is_longer_than_0_lines_The_rest_will_be_skipped: &str =
    "The output is longer than {0} lines. The rest will be skipped.";
pub const Index: &str = "Index";
pub const Missing_vector_opening_bracket: &str = "Missing vector opening bracket \"[\".";
pub const Bracket_mismatch: &str = "Bracket mismatch.";
pub const _must_be_scalar: &str = " must be scalar.";
pub const Index_out_of_range_0: &str = "Index out of range: {0}.";
pub const Cannot_assign_vector_to_scalar: &str = "Cannot assign vector to scalar.";
pub const Missing_vector_closing_bracket: &str = "Missing vector closing bracket \"]\".";
pub const _must_be_vector: &str = " must be vector.";
pub const _must_be_positive_integer: &str = " must be positive integer.";
pub const Step_cannot_be_zero: &str = "Step cannot be zero.";
pub const Vector_size_cannot_exceed_0: &str = "Vector size cannot exceed {0}.";
pub const Cross_product_is_definedonly_for_vectors_with_2_and_3_elements: &str =
    "Cross product is defined only for vectors with 2 and 3 elements.";
pub const Function_0_is_not_defined_in_complex_mode: &str = "Function {0} is not defined in complex mode.";
pub const Matrix_dimensions_do_not_match: &str = "Matrix dimensions do not match.";
pub const Matrix_must_be_square: &str = "Matrix must be square.";
pub const The_Lp_norm_argument_must_be_p_ge_1: &str = "The Lp norm argument must be p ≥ 1.";
pub const Matrix_must_be_symmetric: &str = "Matrix must be symmetric.";
pub const Matrix_is_not_positive_definite: &str = "Matrix is not positive definite.";
pub const Matrix_is_singular: &str = "Matrix is singular.";
pub const Matrix_is_singular_or_close_to_singular: &str = "Matrix is singular or close to singular.";
pub const Vectors_and_matrices_are_not_supported_in_complex_mode: &str =
    "Vectors and matrices are not supported in complex mode.";
pub const Cannot_interpolate_with_non_scalar_value: &str = "Cannot interpolate with non-scalar value.";
pub const Jacobi_iteration_failed: &str = "Jacobi iteration failed.";
pub const Invalid_operand_0: &str = "Invalid operand: {0}.";
pub const Invalid_argument_0: &str = "Invalid argument: {0}.";
pub const The_argument_is_not_an_eigenvalue_0: &str = "The argument is not an eigenvalue: {0}.";
pub const The_QL_algorithm_failed_to_converge: &str = "The QL algorithm failed to converge.";
pub const Matrix_size_cannot_exceed_0: &str = "Matrix size cannot exceed {0}.";
pub const The_number_of_matrix_rows_must_be_greater_than_or_equal_to_the_number_of_columns: &str =
    "The number of matrix rows must be greater than or equal to the number of columns";
pub const Count: &str = "Count";
pub const Number_of_rows: &str = "Number of rows";
pub const Number_of_columns: &str = "Number of columns";
pub const Argument: &str = "Argument";
pub const Result: &str = "Result";
pub const Variable: &str = "Variable";
pub const Index_target: &str = "Index target";
pub const Invalid_item: &str = "Invalid item";
pub const Function_definition_not_allowed_in_solver_block: &str =
    "Function definition not allowed in solver or expression block.";
pub const Brackets_not_closed: &str = "Brackets not closed.";
pub const _must_be_real: &str = " must be real.";
pub const Limit: &str = "Limit";
pub const Value: &str = "Value";
pub const Invalid_format_string_0: &str = "Invalid format string: {0}.";
pub const Invalid_keyword_0: &str = "Invalid keyword: {0}.";
pub const Invalid_JSON_in_0: &str = "Invalid JSON in {0}.";
pub const _must_be_matrix: &str = " must be matrix.";
pub const The_variable_is_not_a_matrix_0: &str = "The variable is not a matrix: {0}.";
pub const File_not_found_0: &str = "File not found: {0}.";
pub const Missing_file_name: &str = "Missing file name.";
pub const Invalid_type_0: &str = "Invalid Type: {0}.";
pub const Path_not_found_0: &str = "Path not found: {0}.";
pub const TheMatrixIsIllConditioned: &str =
    "The matrix is ill-conditioned. Try with lower precision (Tol) or direct Cholesky solver.";
pub const ThePCGSolverFailedToConvergeIn_0_Iterations: &str =
    "The PCG solver failed to converge in {0} iterations. Try with lower precision (Tol) or direct Cholesky solver.";
pub const ThisFileFormatIsNotSupported0: &str = "This file format is not supported: {0}.";
pub const MustBeHpMatrix: &str = " must be hp matrix.";
pub const MustBeHpVector: &str = " must be hp vector.";
pub const TheInputMatrixForFFTMustHaveOneRowForRealAndTwoRowsForComplexData: &str =
    "The input matrix for FFT must have one row for real and two rows for complex data.";
pub const MissingMatrixIndexIn0: &str = "Missing matrix index in {0}.";
pub const CounterMustBeASingleVariableName: &str = "Counter must be a single variable name.";
pub const InvalidNumberOfParameters: &str = "Invalid number of parameters.";
pub const InvalidNumberOfIndexes: &str = "Invalid number of indexes.";
pub const PossiblyInfiniteLoopDetected: &str = "Possibly infinite loop detected, n > {0} iterations.";
pub const CannotModifyConstant: &str = "Cannot modify constant \"{0}\".";
pub const Improper_format_for_UI_keyword_Missing_closing_brace: &str =
    "Improper format for #UI keyword. Missing closing brace '}'.";
pub const The_UI_keyword_requires_a_variable_assignment: &str = "The #UI keyword requires a variable assignment.";
pub const UI_expressions_require_allowExpression: &str =
    "#UI does not support expressions unless 'allowExpression' is true.";
pub const Only_numbers_are_supported_by_the_UI_keyword: &str = "Only numbers are supported by the #UI keyword.";
pub const The_UI_0_requires_both_keys_and_values_arrays: &str = "#UI {0} requires both 'keys' and 'values' arrays.";
pub const The_UI_0_keys_and_values_arrays_must_have_the_same_length: &str =
    "#UI {0} has {1} keys but {2} values, the 'keys' and 'values' arrays must have the same length.";
pub const The_UI_type_0_is_not_recognized_expected_one_of_1: &str =
    "'{0}' is not a recognized #UI type, expected one of {1}.";
pub const The_UI_0_must_not_be_negative: &str = "#UI '{0}' must not be negative.";
pub const The_UI_0_has_1_entries_but_the_grid_has_2_3: &str = "#UI '{0}' has {1} entries but the grid has {2} {3}.";
pub const The_UI_0_does_not_apply_to_1: &str =
    "#UI '{0}' does not apply to a {1}, which always replaces the whole value.";
pub const The_UI_datagrid_requires_a_vector_or_matrix_value: &str = "#UI 'datagrid' requires a vector or matrix value.";
pub const The_UI_0_cannot_hold_a_vector_or_matrix_value: &str =
    "#UI '{0}' cannot hold a vector or matrix value. Use a datagrid.";
pub const The_UI_0_only_applies_to_a_datagrid: &str = "#UI '{0}' only applies to a datagrid.";
pub const The_UI_0_only_applies_to_an_entry: &str = "#UI '{0}' only applies to an entry.";
pub const The_UI_min_must_not_be_above_max: &str = "#UI 'min' ({0}) must not be above 'max' ({1}).";
pub const The_UI_step_must_be_above_zero: &str = "#UI 'step' must be above zero.";
pub const The_UI_value_0_is_outside_1_to_2: &str = "{0} is outside the range {1} to {2}.";
pub const The_UI_width_must_be_a_number_or_a_percentage: &str =
    "#UI 'width' must be a number of pixels or a percentage, e.g. \"75%\".";
pub const Datagrid_size_cannot_exceed_0_cells: &str =
    "#UI datagrid size cannot exceed {0} cells. Use #Read to import larger data sets from a file instead.";
pub const Embedded_data_cannot_exceed_0_MB: &str = "Data embedded in a compiled worksheet cannot exceed {0} MB.";
pub const A_UI_value_has_the_wrong_type: &str = "A #UI value has the wrong type.";
pub const Missing_path_value_0: &str = "{0} requires a path.";
pub const Duplicate_path_declaration_0: &str = "{0} is already declared in this document.";
pub const Path_root_not_declared_0_1: &str = "{0} is not declared. Add {1} before this line.";
pub const Path_root_folder_not_found_0_1: &str = "{0} folder not found: {1}.";
