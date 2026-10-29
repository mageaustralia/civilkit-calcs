//! Error constructors, generated from Calcpad.Core/Exceptions.cs by tools/gen_exceptions.py.
//! Function names keep the C# names so call sites read the same as upstream.
#![allow(non_snake_case, dead_code)]

use crate::math_parser_exception::MathParserException;
use crate::messages;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Items {
    Index,
    Count,
    NumRows,
    NumCols,
    Argument,
    Result,
    Variable,
    IndexTarget,
    Limit,
    Value,
}

fn item_to_string(item: Items) -> &'static str {
    match item {
        Items::Index => messages::Index,
        Items::Count => messages::Count,
        Items::NumRows => messages::Number_of_rows,
        Items::NumCols => messages::Number_of_columns,
        Items::Argument => messages::Argument,
        Items::Result => messages::Result,
        Items::Variable => messages::Variable,
        Items::IndexTarget => messages::Index_target,
        Items::Limit => messages::Limit,
        Items::Value => messages::Value,
    }
}

pub fn IncompleteExpression() -> MathParserException {
    MathParserException::coded("Incomplete_expression", messages::Incomplete_expression, &[])
}

pub fn MissingLeftBracket() -> MathParserException {
    MathParserException::coded("Missing_left_bracket", messages::Missing_left_bracket, &[])
}

pub fn MissingRightBracket() -> MathParserException {
    MathParserException::coded("Missing_right_bracket", messages::Missing_right_bracket, &[])
}

pub fn InvalidSymbol(c: char) -> MathParserException {
    MathParserException::coded("Invalid_symbol_0", messages::Invalid_symbol_0, &[&c.to_string()])
}

pub fn InvalidUnits(s: &str) -> MathParserException {
    MathParserException::coded("Invalid_units_0", messages::Invalid_units_0, &[s])
}

pub fn InvalidLiteral(s: &str, literal: &str) -> MathParserException {
    MathParserException::coded(
        "Cannot_evaluate_0_as_1",
        messages::Cannot_evaluate_0_as_1,
        &[s, literal],
    )
}

pub fn InvalidOperator(c: char) -> MathParserException {
    MathParserException::coded("Invalid_operator_0", messages::Invalid_operator_0, &[&c.to_string()])
}

pub fn PowerNotUnitless() -> MathParserException {
    MathParserException::coded("Power_must_be_unitless", messages::Power_must_be_unitless, &[])
}

pub fn ResultIsNotUnits() -> MathParserException {
    MathParserException::coded(
        "The_expression_on_the_right_does_not_evaluate_to_units",
        messages::The_expression_on_the_right_does_not_evaluate_to_units,
        &[],
    )
}

pub fn CannotEvaluateFunction(s: &str) -> MathParserException {
    MathParserException::coded(
        "Cannot_evaluate_the_function_f_for_v_equals_0",
        messages::Cannot_evaluate_the_function_f_for_v_equals_0,
        &[s],
    )
}

pub fn FunctionNotDefined(s: &str) -> MathParserException {
    MathParserException::coded(
        "The_function_f_is_not_defined_for_v_equals_0",
        messages::The_function_f_is_not_defined_for_v_equals_0,
        &[s],
    )
}

pub fn InconsistentUnits(u1: &str, u2: &str) -> MathParserException {
    MathParserException::coded(
        "Inconsistent_units_0_and_1",
        messages::Inconsistent_units_0_and_1,
        &[u1, u2],
    )
}

pub fn IterationLimits(l1: &str, l2: &str) -> MathParserException {
    MathParserException::coded("Limits_out_of_range_0_1", messages::Limits_out_of_range_0_1, &[l1, l2])
}

pub fn InvalidUnitsFunction(function: &str, unit: &str) -> MathParserException {
    MathParserException::coded(
        "Invalid_units_for_function_0_1",
        messages::Invalid_units_for_function_0_1,
        &[function, unit],
    )
}

pub fn RootUnitless() -> MathParserException {
    MathParserException::coded(
        "Root_index_must_be_unitless",
        messages::Root_index_must_be_unitless,
        &[],
    )
}

pub fn RootComplex() -> MathParserException {
    MathParserException::coded(
        "Root_index_cannot_be_a_complex_number",
        messages::Root_index_cannot_be_a_complex_number,
        &[],
    )
}

pub fn RootInteger() -> MathParserException {
    MathParserException::coded(
        "Root_index_must_be_integer_more_than_1",
        messages::Root_index_must_be_integer_more_than_1,
        &[],
    )
}

pub fn FactorialArgumentOutOfRange() -> MathParserException {
    MathParserException::coded(
        "Argument_out_of_range_for_n_factorial",
        messages::Argument_out_of_range_for_n_factorial,
        &[],
    )
}

pub fn FactorialArgumentUnitless() -> MathParserException {
    MathParserException::coded(
        "The_argument_of_n_factorial_must_be_unitless",
        messages::The_argument_of_n_factorial_must_be_unitless,
        &[],
    )
}

pub fn FactorialArgumentPositiveInteger() -> MathParserException {
    MathParserException::coded(
        "The_argument_of_n_factorial_must_be_a_positive_integer",
        messages::The_argument_of_n_factorial_must_be_a_positive_integer,
        &[],
    )
}

pub fn FactorialArgumentComplex() -> MathParserException {
    MathParserException::coded(
        "The_argument_of_n_factorial_cannot_be_complex",
        messages::The_argument_of_n_factorial_cannot_be_complex,
        &[],
    )
}

pub fn RemainderUnits(u1: &str, u2: &str) -> MathParserException {
    MathParserException::coded(
        "Cannot_evaluate_remainder_0_1_Denominator_must_be_unitless",
        messages::Cannot_evaluate_remainder_0_1_Denominator_must_be_unitless,
        &[u1, u2],
    )
}

pub fn BothValuesInteger() -> MathParserException {
    MathParserException::coded(
        "Both_values_must_be_integers",
        messages::Both_values_must_be_integers,
        &[],
    )
}

pub fn VariableNotExist(name: &str) -> MathParserException {
    MathParserException::coded(
        "Variable_0_does_not_exist",
        messages::Variable_0_does_not_exist,
        &[name],
    )
}

pub fn UnitNotExist(name: &str) -> MathParserException {
    MathParserException::coded("Unit_0_does_not_exist", messages::Unit_0_does_not_exist, &[name])
}

pub fn InteruptedByUser() -> MathParserException {
    MathParserException::coded("Interrupted_by_user", messages::Interrupted_by_user, &[])
}

pub fn CalculationsNotActive() -> MathParserException {
    MathParserException::coded("Calculations_not_active", messages::Calculations_not_active, &[])
}

pub fn ExpressionEmpty() -> MathParserException {
    MathParserException::coded("Expression_is_empty", messages::Expression_is_empty, &[])
}

pub fn MissingOperand() -> MathParserException {
    MathParserException::coded("Missing_operand", messages::Missing_operand, &[])
}

pub fn InvalidFunction(s: &str) -> MathParserException {
    MathParserException::coded("Invalid_function_0", messages::Invalid_function_0, &[s])
}

pub fn InvalidVariableName(s: &str) -> MathParserException {
    MathParserException::coded("Invalid_variable_name_0", messages::Invalid_variable_name_0, &[s])
}

pub fn CannotEvaluateAsType(literal: &str, type_: &str) -> MathParserException {
    MathParserException::coded(
        "Cannot_evaluate_0_as_1",
        messages::Cannot_evaluate_0_as_1,
        &[literal, type_],
    )
}

pub fn StackLeak() -> MathParserException {
    MathParserException::coded(
        "Stack_memory_leak_Invalid_expression",
        messages::Stack_memory_leak_Invalid_expression,
        &[],
    )
}

pub fn StackEmpty() -> MathParserException {
    MathParserException::coded(
        "Stack_empty_Invalid_expression",
        messages::Stack_empty_Invalid_expression,
        &[],
    )
}

pub fn UndefinedInputField() -> MathParserException {
    MathParserException::coded("Undefined_input_field", messages::Undefined_input_field, &[])
}

pub fn UndefinedVariableOrUnits(s: &str) -> MathParserException {
    MathParserException::coded(
        "Undefined_variable_or_units_0",
        messages::Undefined_variable_or_units_0,
        &[s],
    )
}

pub fn ErrorEvaluatingAsFunction(s: &str) -> MathParserException {
    MathParserException::coded(
        "Error_evaluating_0_as_function",
        messages::Error_evaluating_0_as_function,
        &[s],
    )
}

pub fn ErrorEvaluatingAsFunctionOrOperator(s: &str) -> MathParserException {
    MathParserException::coded(
        "Error_evaluating_0_as_function_or_operator",
        messages::Error_evaluating_0_as_function_or_operator,
        &[s],
    )
}

pub fn CannotRewriteUnits(s: &str) -> MathParserException {
    MathParserException::coded(
        "Cannot_rewrite_existing_units_0",
        messages::Cannot_rewrite_existing_units_0,
        &[s],
    )
}

pub fn InconsistentTargetUnits(sourceUnits: &str, targetUnits: &str) -> MathParserException {
    MathParserException::coded(
        "The_calculated_units_0_are_inconsistent_with_the_target_units_1",
        messages::The_calculated_units_0_are_inconsistent_with_the_target_units_1,
        &[sourceUnits, targetUnits],
    )
}

pub fn InvalidCharacter(c: char) -> MathParserException {
    MathParserException::coded(
        "Invalid_character_exception",
        messages::Invalid_character_exception,
        &[&c.to_string()],
    )
}

pub fn ImproperAssignment() -> MathParserException {
    MathParserException::coded(
        "Improper_use_of_the_assignment_operator_equals",
        messages::Improper_use_of_the_assignment_operator_equals,
        &[],
    )
}

pub fn MissingLeftSolverBracket() -> MathParserException {
    MathParserException::coded(
        "Missing_left_bracket_in_solver_command",
        messages::Missing_left_bracket_in_solver_command,
        &[],
    )
}

pub fn MissingRightSolverBracket() -> MathParserException {
    MathParserException::coded(
        "Missing_right_bracket_in_solver_command",
        messages::Missing_right_bracket_in_solver_command,
        &[],
    )
}

pub fn InvalidMacro(s: &str) -> MathParserException {
    MathParserException::coded("Invalid_macro_identifier_0", messages::Invalid_macro_identifier_0, &[s])
}

pub fn InvalidSolver(s: &str) -> MathParserException {
    MathParserException::coded(
        "Invalid_solver_command_definition_0",
        messages::Invalid_solver_command_definition_0,
        &[s],
    )
}

pub fn ErrorParsingUnits(s: &str) -> MathParserException {
    MathParserException::coded("Error_parsing_0_as_units", messages::Error_parsing_0_as_units, &[s])
}

pub fn ErrorParsingNumber(s: &str) -> MathParserException {
    MathParserException::coded("Error_parsing_0_as_number", messages::Error_parsing_0_as_number, &[s])
}

pub fn MissingDelimiter(delimiter: char, script: &str) -> MathParserException {
    MathParserException::coded(
        "Missing_delimiter_0_in_solver_command_1",
        messages::Missing_delimiter_0_in_solver_command_1,
        &[&delimiter.to_string(), script],
    )
}

pub fn MultipleAssignments(s: &str) -> MathParserException {
    MathParserException::coded(
        "More_than_one_operator_equals_in_0",
        messages::More_than_one_operator_equals_in_0,
        &[s],
    )
}

pub fn NotConstantExpression(s: &str) -> MathParserException {
    MathParserException::coded(
        "The_expression_on_the_right_side_must_be_constant_0",
        messages::The_expression_on_the_right_side_must_be_constant_0,
        &[s],
    )
}

pub fn InconsistentUnits1(variable: &str, units: &str) -> MathParserException {
    MathParserException::coded(
        "Inconsistent_units_for_0_equals_1",
        messages::Inconsistent_units_for_0_equals_1,
        &[variable, units],
    )
}

pub fn InconsistentUnits2(variable: &str, u1: &str, u2: &str) -> MathParserException {
    MathParserException::coded(
        "Inconsistent_units_for_0_equals_1_2",
        messages::Inconsistent_units_for_0_equals_1_2,
        &[variable, u1, u2],
    )
}

pub fn NoSolution(s: &str) -> MathParserException {
    MathParserException::coded("No_solution_for_0", messages::No_solution_for_0, &[s])
}

pub fn RecursionNotAllowed(s: &str) -> MathParserException {
    MathParserException::coded(
        "Recursion_is_not_allowed_in_function_definition_0",
        messages::Recursion_is_not_allowed_in_function_definition_0,
        &[s],
    )
}

pub fn AssignmentPreceded() -> MathParserException {
    MathParserException::coded(
        "The_assignment_equals_must_be_preceded_by_custom_function_or_variable",
        messages::The_assignment_equals_must_be_preceded_by_custom_function_or_variable,
        &[],
    )
}

pub fn AssignmentNotFirst() -> MathParserException {
    MathParserException::coded(
        "Assignment_equals_must_be_the_first_operator_in_the_expression",
        messages::Assignment_equals_must_be_the_first_operator_in_the_expression,
        &[],
    )
}

pub fn InvalidSyntax(s: &str) -> MathParserException {
    MathParserException::coded("Invalid_syntax_0", messages::Invalid_syntax_0, &[s])
}

pub fn InvalidSyntax2(s1: &str, s2: &str) -> MathParserException {
    MathParserException::coded("Invalid_syntax_0_1", messages::Invalid_syntax_0_1, &[s1, s2])
}

pub fn UnexpectedDelimiter() -> MathParserException {
    MathParserException::coded("Unexpected_delimiter", messages::Unexpected_delimiter, &[])
}

pub fn InvalidNumberOfArguments() -> MathParserException {
    MathParserException::coded(
        "Invalid_number_of_arguments",
        messages::Invalid_number_of_arguments,
        &[],
    )
}

pub fn ResultNotReal(s: &str) -> MathParserException {
    MathParserException::coded(
        "The_result_is_not_a_real_number_0",
        messages::The_result_is_not_a_real_number_0,
        &[s],
    )
}

pub fn MissingFunctionParameter() -> MathParserException {
    MathParserException::coded(
        "Missing_parameter_in_function_definition",
        messages::Missing_parameter_in_function_definition,
        &[],
    )
}

pub fn MissingFunctionDelimiter() -> MathParserException {
    MathParserException::coded(
        "Missing_delimiter_in_function_definition",
        messages::Missing_delimiter_in_function_definition,
        &[],
    )
}

pub fn InvalidFunctionToken(name: &str) -> MathParserException {
    MathParserException::coded(
        "Invalid_token_in_function_definition_0",
        messages::Invalid_token_in_function_definition_0,
        &[name],
    )
}

pub fn CircularReference(name: &str) -> MathParserException {
    MathParserException::coded(
        "Circular_reference_detected_for_function_0",
        messages::Circular_reference_detected_for_function_0,
        &[name],
    )
}

pub fn CircularMacroReference(name: &str) -> MathParserException {
    MathParserException::coded(
        "Circular_reference_detected_for_macro_0",
        messages::Circular_reference_detected_for_macro_0,
        &[name],
    )
}

pub fn InvalidFunctionDefinition() -> MathParserException {
    MathParserException::coded(
        "Invalid_function_definition_exception",
        messages::Invalid_function_definition_exception,
        &[],
    )
}

pub fn ArgumentOutOfRange(function: &str) -> MathParserException {
    MathParserException::coded(
        "Argument_out_of_range_for_0_x",
        messages::Argument_out_of_range_for_0_x,
        &[function],
    )
}

pub fn ConditionEmpty() -> MathParserException {
    MathParserException::coded("Condition_cannot_be_empty", messages::Condition_cannot_be_empty, &[])
}

pub fn ConditionNotInitialized() -> MathParserException {
    MathParserException::coded(
        "Condition_block_not_initialized_with_if",
        messages::Condition_block_not_initialized_with_if,
        &[],
    )
}

pub fn DuplicateElse() -> MathParserException {
    MathParserException::coded(
        "Duplicate_else_in_condition_block",
        messages::Duplicate_else_in_condition_block,
        &[],
    )
}

pub fn ElseIfAfterElse() -> MathParserException {
    MathParserException::coded(
        "else_if_is_not_allowed_after_else_in_condition_block",
        messages::else_if_is_not_allowed_after_else_in_condition_block,
        &[],
    )
}

pub fn ConditionComplex() -> MathParserException {
    MathParserException::coded(
        "Condition_cannot_evaluate_to_a_complex_number",
        messages::Condition_cannot_evaluate_to_a_complex_number,
        &[],
    )
}

pub fn ConditionResultInvalid(s: &str) -> MathParserException {
    MathParserException::coded(
        "Condition_result_is_invalid_0",
        messages::Condition_result_is_invalid_0,
        &[s],
    )
}

pub fn DuplicateMacroParameters(s: &str) -> MathParserException {
    MathParserException::coded(
        "Duplicate_macro_parameter_names_0_and_1",
        messages::Duplicate_macro_parameter_names_0_and_1,
        &[s, s],
    )
}

pub fn UndefinedMacro(s: &str) -> MathParserException {
    MathParserException::coded("Macro_not_defined_0", messages::Macro_not_defined_0, &[s])
}

pub fn RequiredParameterAfterOptional(name: &str) -> MathParserException {
    MathParserException::coded(
        "Required_parameter_after_optional_0",
        messages::Required_parameter_after_optional_0,
        &[name],
    )
}

pub fn UnknownKeywordArgument(name: &str) -> MathParserException {
    MathParserException::coded(
        "Unknown_keyword_argument_0",
        messages::Unknown_keyword_argument_0,
        &[name],
    )
}

pub fn DuplicateArgument(name: &str) -> MathParserException {
    MathParserException::coded("Duplicate_argument_0", messages::Duplicate_argument_0, &[name])
}

pub fn MissingMapItem(s: &str) -> MathParserException {
    MathParserException::coded(
        "Missing_0_in_surface_map_command",
        messages::Missing_0_in_surface_map_command,
        &[s],
    )
}

pub fn PlotLimitsIdentical() -> MathParserException {
    MathParserException::coded(
        "The_limits_of_plot_area_are_identical",
        messages::The_limits_of_plot_area_are_identical,
        &[],
    )
}

pub fn ErrorWritingPngFile(path: &str) -> MathParserException {
    MathParserException::coded(
        "Error_writing_a_png_file_to_0",
        messages::Error_writing_a_png_file_to_0,
        &[path],
    )
}

pub fn ErrorWritingSvgFile(path: &str) -> MathParserException {
    MathParserException::coded(
        "Error_writing_a_svg_file_to_0",
        messages::Error_writing_a_svg_file_to_0,
        &[path],
    )
}

pub fn ErrorConvertingPngToBase64() -> MathParserException {
    MathParserException::coded(
        "Error_converting_png_to_Base64",
        messages::Error_converting_png_to_Base64,
        &[],
    )
}

pub fn InconsistentUnitsOperation(ua: &str, op: char, ub: &str) -> MathParserException {
    MathParserException::coded(
        "Inconsistent_units_0_1_2",
        messages::Inconsistent_units_0_1_2,
        &[ua, &op.to_string(), ub],
    )
}

pub fn UnitsToComplexPower() -> MathParserException {
    MathParserException::coded(
        "Units_cannon_be_raised_to_complex_power",
        messages::Units_cannon_be_raised_to_complex_power,
        &[],
    )
}

pub fn CannotEvaluateRemainder(ua: &str, ub: &str) -> MathParserException {
    MathParserException::coded(
        "Cannot_evaluate_remainder_0_1_The_denominator_must_be_unitless",
        messages::Cannot_evaluate_remainder_0_1_The_denominator_must_be_unitless,
        &[ua, ub],
    )
}

pub fn MissingVectorOpeningBracket() -> MathParserException {
    MathParserException::coded(
        "Missing_vector_opening_bracket",
        messages::Missing_vector_opening_bracket,
        &[],
    )
}

pub fn BracketMismatch() -> MathParserException {
    MathParserException::coded("Bracket_mismatch", messages::Bracket_mismatch, &[])
}

pub fn IndexOutOfRange(index: &str) -> MathParserException {
    MathParserException::coded("Index_out_of_range_0", messages::Index_out_of_range_0, &[index])
}

pub fn CannotAssignVectorToScalar() -> MathParserException {
    MathParserException::coded(
        "Cannot_assign_vector_to_scalar",
        messages::Cannot_assign_vector_to_scalar,
        &[],
    )
}

pub fn MissingVectorClosingBracket() -> MathParserException {
    MathParserException::coded(
        "Missing_vector_closing_bracket",
        messages::Missing_vector_closing_bracket,
        &[],
    )
}

pub fn MustBeScalar(item: Items) -> MathParserException {
    MathParserException::new(format!("{}{}", item_to_string(item), messages::_must_be_scalar))
}

pub fn MustBeReal(item: Items) -> MathParserException {
    MathParserException::new(format!("{}{}", item_to_string(item), messages::_must_be_real))
}

pub fn MustBePositiveInteger(item: Items) -> MathParserException {
    MathParserException::new(format!(
        "{}{}",
        item_to_string(item),
        messages::_must_be_positive_integer
    ))
}

pub fn MustBeMatrix(item: Items) -> MathParserException {
    MathParserException::new(format!("{}{}", item_to_string(item), messages::_must_be_matrix))
}

pub fn MustBeVector(item: Items) -> MathParserException {
    MathParserException::new(format!("{}{}", item_to_string(item), messages::_must_be_vector))
}

pub fn StepCannotBeZero() -> MathParserException {
    MathParserException::coded("Step_cannot_be_zero", messages::Step_cannot_be_zero, &[])
}

pub fn VectorSizeLimit() -> MathParserException {
    MathParserException::coded(
        "Vector_size_cannot_exceed_0",
        messages::Vector_size_cannot_exceed_0,
        &["10000000"],
    )
}

pub fn CrossProductVectorDimensions() -> MathParserException {
    MathParserException::coded(
        "Cross_product_is_definedonly_for_vectors_with_2_and_3_elements",
        messages::Cross_product_is_definedonly_for_vectors_with_2_and_3_elements,
        &[],
    )
}

pub fn FunctionOnlyRealMode(name: &str) -> MathParserException {
    MathParserException::coded(
        "Function_0_is_not_defined_in_complex_mode",
        messages::Function_0_is_not_defined_in_complex_mode,
        &[name],
    )
}

pub fn MatrixDimensions() -> MathParserException {
    MathParserException::coded(
        "Matrix_dimensions_do_not_match",
        messages::Matrix_dimensions_do_not_match,
        &[],
    )
}

pub fn MatrixNotSquare() -> MathParserException {
    MathParserException::coded("Matrix_must_be_square", messages::Matrix_must_be_square, &[])
}

pub fn InvalidLpNormArgument() -> MathParserException {
    MathParserException::coded(
        "The_Lp_norm_argument_must_be_p_ge_1",
        messages::The_Lp_norm_argument_must_be_p_ge_1,
        &[],
    )
}

pub fn MatrixMustBeSymmetric() -> MathParserException {
    MathParserException::coded("Matrix_must_be_symmetric", messages::Matrix_must_be_symmetric, &[])
}

pub fn MatrixNotPositiveDefinite() -> MathParserException {
    MathParserException::coded(
        "Matrix_is_not_positive_definite",
        messages::Matrix_is_not_positive_definite,
        &[],
    )
}

pub fn MatrixSingular() -> MathParserException {
    MathParserException::coded("Matrix_is_singular", messages::Matrix_is_singular, &[])
}

pub fn MatrixCloseToSingular() -> MathParserException {
    MathParserException::coded(
        "Matrix_is_singular_or_close_to_singular",
        messages::Matrix_is_singular_or_close_to_singular,
        &[],
    )
}

pub fn ComplexVectorsAndMatricesNotSupported() -> MathParserException {
    MathParserException::coded(
        "Vectors_and_matrices_are_not_supported_in_complex_mode",
        messages::Vectors_and_matrices_are_not_supported_in_complex_mode,
        &[],
    )
}

pub fn CannotInterpolateWithNonScalarValue() -> MathParserException {
    MathParserException::coded(
        "Cannot_interpolate_with_non_scalar_value",
        messages::Cannot_interpolate_with_non_scalar_value,
        &[],
    )
}

pub fn JacobiFailed() -> MathParserException {
    MathParserException::coded("Jacobi_iteration_failed", messages::Jacobi_iteration_failed, &[])
}

pub fn InvalidOperand(value: &str) -> MathParserException {
    MathParserException::coded("Invalid_operand_0", messages::Invalid_operand_0, &[value])
}

pub fn InvalidArgument(value: &str) -> MathParserException {
    MathParserException::coded("Invalid_argument_0", messages::Invalid_argument_0, &[value])
}

pub fn ArgumentNotEigenvalue(value: &str) -> MathParserException {
    MathParserException::coded(
        "The_argument_is_not_an_eigenvalue_0",
        messages::The_argument_is_not_an_eigenvalue_0,
        &[value],
    )
}

pub fn QLAlgorithmFailed() -> MathParserException {
    MathParserException::coded(
        "The_QL_algorithm_failed_to_converge",
        messages::The_QL_algorithm_failed_to_converge,
        &[],
    )
}

pub fn MatirixSizeLimit() -> MathParserException {
    MathParserException::coded(
        "Matrix_size_cannot_exceed_0",
        messages::Matrix_size_cannot_exceed_0,
        &["1000000"],
    )
}

pub fn EmbeddedDataSizeLimit() -> MathParserException {
    MathParserException::coded(
        "Embedded_data_cannot_exceed_0_MB",
        messages::Embedded_data_cannot_exceed_0_MB,
        &["10"],
    )
}

pub fn DatagridSizeLimit() -> MathParserException {
    MathParserException::coded(
        "Datagrid_size_cannot_exceed_0_cells",
        messages::Datagrid_size_cannot_exceed_0_cells,
        &["100000"],
    )
}

pub fn MatrixNotHigh() -> MathParserException {
    MathParserException::coded(
        "The_number_of_matrix_rows_must_be_greater_than_or_equal_to_the_number_of_columns",
        messages::The_number_of_matrix_rows_must_be_greater_than_or_equal_to_the_number_of_columns,
        &[],
    )
}

pub fn FunctionDefinitionInSolver() -> MathParserException {
    MathParserException::coded(
        "Function_definition_not_allowed_in_solver_block",
        messages::Function_definition_not_allowed_in_solver_block,
        &[],
    )
}

pub fn InvalidFormatString(s: &str) -> MathParserException {
    MathParserException::coded("Invalid_format_string_0", messages::Invalid_format_string_0, &[s])
}

pub fn VariableNotMatrix(name: &str) -> MathParserException {
    MathParserException::coded(
        "The_variable_is_not_a_matrix_0",
        messages::The_variable_is_not_a_matrix_0,
        &[name],
    )
}

pub fn FileNotFound(path: &str) -> MathParserException {
    MathParserException::coded("File_not_found_0", messages::File_not_found_0, &[path])
}

pub fn MissingFileName() -> MathParserException {
    MathParserException::coded("Missing_file_name", messages::Missing_file_name, &[])
}

pub fn InvalidType(type_: char) -> MathParserException {
    MathParserException::coded("Invalid_type_0", messages::Invalid_type_0, &[&type_.to_string()])
}

pub fn PathNotFound(path: &str) -> MathParserException {
    MathParserException::coded("Path_not_found_0", messages::Path_not_found_0, &[path])
}

pub fn FileFormatNotSupported(ext: &str) -> MathParserException {
    MathParserException::coded(
        "ThisFileFormatIsNotSupported0",
        messages::ThisFileFormatIsNotSupported0,
        &[ext],
    )
}

pub fn MustBeHpMatrix(item: Items) -> MathParserException {
    MathParserException::new(format!("{}{}", item_to_string(item), messages::MustBeHpMatrix))
}

pub fn MustBeHpVector(item: Items) -> MathParserException {
    MathParserException::new(format!("{}{}", item_to_string(item), messages::MustBeHpVector))
}

pub fn MatrixOneOrTwoRows() -> MathParserException {
    MathParserException::coded(
        "TheInputMatrixForFFTMustHaveOneRowForRealAndTwoRowsForComplexData",
        messages::TheInputMatrixForFFTMustHaveOneRowForRealAndTwoRowsForComplexData,
        &[],
    )
}

pub fn MissingMatrixIndex(matrix: &str) -> MathParserException {
    MathParserException::coded("MissingMatrixIndexIn0", messages::MissingMatrixIndexIn0, &[matrix])
}

pub fn CounterMustBeASingleVariableName() -> MathParserException {
    MathParserException::coded(
        "CounterMustBeASingleVariableName",
        messages::CounterMustBeASingleVariableName,
        &[],
    )
}

pub fn InvalidNumberOfIndexes() -> MathParserException {
    MathParserException::coded("InvalidNumberOfIndexes", messages::InvalidNumberOfIndexes, &[])
}

pub fn InfiniteLoop(maxCount: i32) -> MathParserException {
    MathParserException::coded(
        "PossiblyInfiniteLoopDetected",
        messages::PossiblyInfiniteLoopDetected,
        &[&maxCount.to_string()],
    )
}

pub fn CannotModifyConstant(name: &str) -> MathParserException {
    MathParserException::coded("CannotModifyConstant", messages::CannotModifyConstant, &[name])
}

pub fn InvalidNumber(s: &str) -> MathParserException {
    InvalidLiteral(s, "number")
}
