package api

import (
	"bobmigrate/models"
	"github.com/gofiber/fiber/v2"
)

// PayrollRequest separates the API payload from the internal database model
type PayrollRequest struct {
	EmpID       string `json:"emp_id"`
	EmpStatus   string `json:"emp_status"`
	HoursWorked int64  `json:"hours_worked"`
	HourlyRate  int64  `json:"hourly_rate_cents"`
}

// PayrollResponse returns the calculated output
type PayrollResponse struct {
	EmpID         string `json:"emp_id"`
	GrossPayCents int64  `json:"gross_pay_cents"`
	Status        string `json:"status"`
}

func SetupRoutes(app *fiber.App) {
	app.Post("/api/v1/payroll", ProcessPayrollHandler)
}

func ProcessPayrollHandler(c *fiber.Ctx) error {
	var req PayrollRequest
	if err := c.BodyParser(&req); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "invalid JSON body",
		})
	}

	if req.HoursWorked < 0 || req.HourlyRate < 0 {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "hours and rates cannot be negative",
		})
	}

	// Map DTO to the modernized business logic model
	record := &models.EmployeeRecord{
		EmpID:       req.EmpID,
		EmpStatus:   req.EmpStatus,
		HoursWorked: req.HoursWorked,
		HourlyRate:  req.HourlyRate,
	}

	// Execute translated legacy logic
	models.CalculatePay(record)

	return c.Status(fiber.StatusOK).JSON(PayrollResponse{
		EmpID:         record.EmpID,
		GrossPayCents: record.GrossPay,
		Status:        "success",
	})
}
