package models_test

import (
	"bobmigrate/models"
	"testing"
)

func TestCalculatePay(t *testing.T) {
	tests := []struct {
		name     string
		input    models.EmployeeRecord
		expected int64 // expected gross pay in cents
	}{
		{
			name: "Happy Path - 40 hours exactly",
			input: models.EmployeeRecord{
				EmpID:       "E0001",
				EmpStatus:   "A",
				HoursWorked: 40,
				HourlyRate:  2500, // $25.00
			},
			expected: 100000, // $1,000.00
		},
		{
			name: "Overtime Trap - 50 hours at 1.5x",
			input: models.EmployeeRecord{
				EmpID:       "E0002",
				EmpStatus:   "A",
				HoursWorked: 50,
				HourlyRate:  2000, // $20.00
			},
			expected: 110000, // (40 * 20) + (10 * 30) = $1,100.00
		},
		{
			name: "Business Logic Gate - Suspended Account",
			input: models.EmployeeRecord{
				EmpID:       "E0003",
				EmpStatus:   "S",
				HoursWorked: 45,
				HourlyRate:  3000, // $30.00
			},
			expected: 0, // Suspended gets zero pay
		},
		{
			name: "Fractional Cent Handling - 10 hours",
			input: models.EmployeeRecord{
				EmpID:       "E0004",
				EmpStatus:   "A",
				HoursWorked: 10,
				HourlyRate:  1550, // $15.50
			},
			expected: 15500, // $155.00
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			record := tt.input // copy struct
			models.CalculatePay(&record)

			if record.GrossPay != tt.expected {
				t.Errorf("expected gross pay %d, got %d", tt.expected, record.GrossPay)
			}
		})
	}
}
