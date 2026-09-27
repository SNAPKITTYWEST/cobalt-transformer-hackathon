package models

// EmployeeRecord represents the 01 level WS-EMPLOYEE-RECORD
type EmployeeRecord struct {
	EmpID       string `json:"emp_id" cobol:"type:string,length:5"`
	EmpStatus   string `json:"emp_status" cobol:"type:string,length:1"`
	HoursWorked int64  `json:"hours_worked" cobol:"type:int,length:2"`
	HourlyRate  int64  `json:"hourly_rate_cents" cobol:"type:money_cents,length:6"`
	GrossPay    int64  `json:"gross_pay_cents" cobol:"type:money_cents,length:9"`
}

// IsSuspended handles the 88 level IS-SUSPENDED condition
func (e *EmployeeRecord) IsSuspended() bool {
	return e.EmpStatus == "S"
}

// CalculatePay translates the 300-CALCULATE-PAY COBOL paragraph
func CalculatePay(record *EmployeeRecord) {
	if record == nil {
		return
	}

	if record.IsSuspended() {
		record.GrossPay = 0
		return
	}

	if record.HoursWorked > 40 {
		basePay := 40 * record.HourlyRate
		overtimeHours := record.HoursWorked - 40

		// Integer math for 1.5x multiplier to prevent IEEE 754 float leaks
		overtimePay := (overtimeHours * record.HourlyRate * 150) / 100

		record.GrossPay = basePay + overtimePay
	} else {
		record.GrossPay = record.HoursWorked * record.HourlyRate
	}
}
