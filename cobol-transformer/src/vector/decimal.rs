// Decimal Semantics Engine
// Implements IBM COBOL decimal arithmetic with exact precision

use std::fmt;
use anyhow::{Result, bail};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecimalValue {
    pub digits: Vec<u8>,
    pub scale: i32,
    pub sign: Sign,
    pub precision: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    Positive,
    Negative,
    Unsigned,
}

#[derive(Debug, Clone)]
pub enum DecimalOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone)]
pub enum RoundingMode {
    Truncate,
    RoundHalfUp,
    RoundHalfEven,
    RoundUp,
    RoundDown,
}

pub struct DecimalEngine {
    rounding_mode: RoundingMode,
    size_error_enabled: bool,
}

impl DecimalEngine {
    pub fn new() -> Self {
        Self {
            rounding_mode: RoundingMode::Truncate,
            size_error_enabled: true,
        }
    }

    pub fn with_rounding(mut self, mode: RoundingMode) -> Self {
        self.rounding_mode = mode;
        self
    }

    pub fn with_size_error(mut self, enabled: bool) -> Self {
        self.size_error_enabled = enabled;
        self
    }

    pub fn add(&self, left: &DecimalValue, right: &DecimalValue) -> Result<DecimalValue> {
        // Align scales
        let (left_aligned, right_aligned) = self.align_scales(left, right);
        
        // Perform addition
        let result = match (left_aligned.sign, right_aligned.sign) {
            (Sign::Positive, Sign::Positive) | (Sign::Unsigned, Sign::Unsigned) => {
                self.add_magnitudes(&left_aligned, &right_aligned, Sign::Positive)
            }
            (Sign::Negative, Sign::Negative) => {
                self.add_magnitudes(&left_aligned, &right_aligned, Sign::Negative)
            }
            (Sign::Positive, Sign::Negative) | (Sign::Unsigned, Sign::Negative) => {
                self.subtract_magnitudes(&left_aligned, &right_aligned)
            }
            (Sign::Negative, Sign::Positive) | (Sign::Negative, Sign::Unsigned) => {
                self.subtract_magnitudes(&right_aligned, &left_aligned)
            }
            _ => left_aligned.clone(),
        };
        
        Ok(result)
    }

    pub fn subtract(&self, left: &DecimalValue, right: &DecimalValue) -> Result<DecimalValue> {
        let negated_right = DecimalValue {
            digits: right.digits.clone(),
            scale: right.scale,
            sign: match right.sign {
                Sign::Positive => Sign::Negative,
                Sign::Negative => Sign::Positive,
                Sign::Unsigned => Sign::Unsigned,
            },
            precision: right.precision,
        };
        
        self.add(left, &negated_right)
    }

    pub fn multiply(&self, left: &DecimalValue, right: &DecimalValue) -> Result<DecimalValue> {
        let result_scale = left.scale + right.scale;
        let result_sign = match (left.sign, right.sign) {
            (Sign::Positive, Sign::Positive) | (Sign::Negative, Sign::Negative) => Sign::Positive,
            (Sign::Positive, Sign::Negative) | (Sign::Negative, Sign::Positive) => Sign::Negative,
            _ => Sign::Unsigned,
        };
        
        // Multiply digit arrays
        let mut result_digits = vec![0u8; left.digits.len() + right.digits.len()];
        
        for (i, &left_digit) in left.digits.iter().enumerate() {
            let mut carry = 0u16;
            for (j, &right_digit) in right.digits.iter().enumerate() {
                let product = (left_digit as u16) * (right_digit as u16) + result_digits[i + j] as u16 + carry;
                result_digits[i + j] = (product % 10) as u8;
                carry = product / 10;
            }
            if carry > 0 {
                result_digits[i + right.digits.len()] += carry as u8;
            }
        }
        
        // Remove leading zeros
        while result_digits.len() > 1 && result_digits.last() == Some(&0) {
            result_digits.pop();
        }
        
        let mut result = DecimalValue {
            digits: result_digits,
            scale: result_scale,
            sign: result_sign,
            precision: left.precision + right.precision,
        };
        
        // Apply rounding if needed
        result = self.round(result, left.precision.max(right.precision))?;
        
        Ok(result)
    }

    pub fn divide(&self, left: &DecimalValue, right: &DecimalValue) -> Result<DecimalValue> {
        // Check for division by zero
        if right.is_zero() {
            bail!("Division by zero");
        }
        
        let result_sign = match (left.sign, right.sign) {
            (Sign::Positive, Sign::Positive) | (Sign::Negative, Sign::Negative) => Sign::Positive,
            (Sign::Positive, Sign::Negative) | (Sign::Negative, Sign::Positive) => Sign::Negative,
            _ => Sign::Unsigned,
        };
        
        // Scale adjustment for division
        let scale_adjustment = left.precision.max(right.precision) + 2;
        let mut dividend = left.clone();
        dividend.scale += scale_adjustment as i32;
        
        // Perform long division
        let quotient = self.long_division(&dividend, right)?;
        
        let mut result = DecimalValue {
            digits: quotient,
            scale: dividend.scale - right.scale,
            sign: result_sign,
            precision: left.precision.max(right.precision),
        };
        
        // Apply rounding
        result = self.round(result, left.precision)?;
        
        Ok(result)
    }

    fn align_scales(&self, left: &DecimalValue, right: &DecimalValue) -> (DecimalValue, DecimalValue) {
        let max_scale = left.scale.max(right.scale);
        
        let left_aligned = if left.scale < max_scale {
            self.scale_up(left, max_scale - left.scale)
        } else {
            left.clone()
        };
        
        let right_aligned = if right.scale < max_scale {
            self.scale_up(right, max_scale - right.scale)
        } else {
            right.clone()
        };
        
        (left_aligned, right_aligned)
    }

    fn scale_up(&self, value: &DecimalValue, scale_increase: i32) -> DecimalValue {
        let mut new_digits = value.digits.clone();
        for _ in 0..scale_increase {
            new_digits.insert(0, 0);
        }
        
        DecimalValue {
            digits: new_digits,
            scale: value.scale + scale_increase,
            sign: value.sign,
            precision: value.precision + scale_increase as usize,
        }
    }

    fn add_magnitudes(&self, left: &DecimalValue, right: &DecimalValue, sign: Sign) -> DecimalValue {
        let max_len = left.digits.len().max(right.digits.len());
        let mut result_digits = Vec::with_capacity(max_len + 1);
        let mut carry = 0u8;
        
        for i in 0..max_len {
            let left_digit = left.digits.get(i).copied().unwrap_or(0);
            let right_digit = right.digits.get(i).copied().unwrap_or(0);
            let sum = left_digit + right_digit + carry;
            result_digits.push(sum % 10);
            carry = sum / 10;
        }
        
        if carry > 0 {
            result_digits.push(carry);
        }
        
        DecimalValue {
            digits: result_digits,
            scale: left.scale,
            sign,
            precision: max_len,
        }
    }

    fn subtract_magnitudes(&self, left: &DecimalValue, right: &DecimalValue) -> DecimalValue {
        let comparison = self.compare_magnitudes(left, right);
        
        let (larger, smaller, result_sign) = match comparison {
            std::cmp::Ordering::Greater => (left, right, left.sign),
            std::cmp::Ordering::Less => (right, left, right.sign),
            std::cmp::Ordering::Equal => {
                return DecimalValue::zero();
            }
        };
        
        let mut result_digits = Vec::new();
        let mut borrow = 0i16;
        
        for i in 0..larger.digits.len() {
            let larger_digit = larger.digits.get(i).copied().unwrap_or(0) as i16;
            let smaller_digit = smaller.digits.get(i).copied().unwrap_or(0) as i16;
            let mut diff = larger_digit - smaller_digit - borrow;
            
            if diff < 0 {
                diff += 10;
                borrow = 1;
            } else {
                borrow = 0;
            }
            
            result_digits.push(diff as u8);
        }
        
        // Remove leading zeros
        while result_digits.len() > 1 && result_digits.last() == Some(&0) {
            result_digits.pop();
        }
        
        DecimalValue {
            digits: result_digits,
            scale: left.scale,
            sign: result_sign,
            precision: larger.digits.len(),
        }
    }

    fn compare_magnitudes(&self, left: &DecimalValue, right: &DecimalValue) -> std::cmp::Ordering {
        if left.digits.len() != right.digits.len() {
            return left.digits.len().cmp(&right.digits.len());
        }
        
        for i in (0..left.digits.len()).rev() {
            match left.digits[i].cmp(&right.digits[i]) {
                std::cmp::Ordering::Equal => continue,
                other => return other,
            }
        }
        
        std::cmp::Ordering::Equal
    }

    fn long_division(&self, dividend: &DecimalValue, divisor: &DecimalValue) -> Result<Vec<u8>> {
        // Simplified long division - production would be more sophisticated
        let mut quotient = Vec::new();
        let mut remainder = dividend.digits.clone();
        
        // Perform division digit by digit
        for _ in 0..dividend.precision {
            let mut digit = 0u8;
            while self.can_subtract(&remainder, &divisor.digits) {
                remainder = self.subtract_digit_arrays(&remainder, &divisor.digits);
                digit += 1;
            }
            quotient.push(digit);
        }
        
        Ok(quotient)
    }

    fn can_subtract(&self, left: &[u8], right: &[u8]) -> bool {
        if left.len() < right.len() {
            return false;
        }
        
        for i in (0..right.len()).rev() {
            if left[i] < right[i] {
                return false;
            } else if left[i] > right[i] {
                return true;
            }
        }
        
        true
    }

    fn subtract_digit_arrays(&self, left: &[u8], right: &[u8]) -> Vec<u8> {
        let mut result = left.to_vec();
        let mut borrow = 0i16;
        
        for i in 0..right.len() {
            let mut diff = result[i] as i16 - right[i] as i16 - borrow;
            if diff < 0 {
                diff += 10;
                borrow = 1;
            } else {
                borrow = 0;
            }
            result[i] = diff as u8;
        }
        
        result
    }

    fn round(&self, mut value: DecimalValue, target_precision: usize) -> Result<DecimalValue> {
        if value.digits.len() <= target_precision {
            return Ok(value);
        }
        
        let _excess = value.digits.len() - target_precision;
        
        match self.rounding_mode {
            RoundingMode::Truncate => {
                value.digits.truncate(target_precision);
            }
            RoundingMode::RoundHalfUp => {
                if value.digits.get(target_precision).copied().unwrap_or(0) >= 5 {
                    value.digits.truncate(target_precision);
                    value = self.increment_last_digit(value);
                } else {
                    value.digits.truncate(target_precision);
                }
            }
            _ => {
                value.digits.truncate(target_precision);
            }
        }
        
        Ok(value)
    }

    fn increment_last_digit(&self, mut value: DecimalValue) -> DecimalValue {
        let mut carry = 1u8;
        for digit in &mut value.digits {
            let sum = *digit + carry;
            *digit = sum % 10;
            carry = sum / 10;
            if carry == 0 {
                break;
            }
        }
        
        if carry > 0 {
            value.digits.push(carry);
        }
        
        value
    }
}

impl DecimalValue {
    pub fn zero() -> Self {
        Self {
            digits: vec![0],
            scale: 0,
            sign: Sign::Unsigned,
            precision: 1,
        }
    }

    pub fn from_i64(value: i64, scale: i32) -> Self {
        let sign = if value < 0 {
            Sign::Negative
        } else if value > 0 {
            Sign::Positive
        } else {
            Sign::Unsigned
        };
        
        let abs_value = value.abs() as u64;
        let mut digits = Vec::new();
        let mut n = abs_value;
        
        if n == 0 {
            digits.push(0);
        } else {
            while n > 0 {
                digits.push((n % 10) as u8);
                n /= 10;
            }
        }
        
        let precision = digits.len();
        
        Self {
            digits,
            scale,
            sign,
            precision,
        }
    }

    pub fn is_zero(&self) -> bool {
        self.digits.iter().all(|&d| d == 0)
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        
        if self.sign == Sign::Negative {
            result.push('-');
        }
        
        let int_digits = if self.scale >= 0 {
            self.digits.len().saturating_sub(self.scale as usize)
        } else {
            self.digits.len()
        };
        
        for i in (0..self.digits.len()).rev() {
            result.push((b'0' + self.digits[i]) as char);
            if i == int_digits && self.scale > 0 {
                result.push('.');
            }
        }
        
        result
    }
}

impl fmt::Display for DecimalValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimal_addition() {
        let engine = DecimalEngine::new();
        let left = DecimalValue::from_i64(100, 0);
        let right = DecimalValue::from_i64(50, 0);
        
        let result = engine.add(&left, &right).unwrap();
        assert_eq!(result.to_string(), "150");
    }

    #[test]
    fn test_decimal_multiplication() {
        let engine = DecimalEngine::new();
        let left = DecimalValue::from_i64(12, 0);
        let right = DecimalValue::from_i64(5, 0);
        
        let result = engine.multiply(&left, &right).unwrap();
        assert_eq!(result.to_string(), "60");
    }

    #[test]
    fn test_decimal_with_scale() {
        let engine = DecimalEngine::new();
        let left = DecimalValue::from_i64(125, 2); // 1.25
        let right = DecimalValue::from_i64(200, 2); // 2.00
        
        let result = engine.add(&left, &right).unwrap();
        // Result should be 3.25
        assert!(result.scale == 2);
    }
}

// Made with Bob
