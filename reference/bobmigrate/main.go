package main

import (
	"log"
	"bobmigrate/api"
	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/logger"
)

func main() {
	app := fiber.New()

	// Add logging middleware to show requests in the terminal during the demo
	app.Use(logger.New())

	api.SetupRoutes(app)

	log.Println("🚀 BobMigrate API running on http://localhost:3000")
	log.Fatal(app.Listen(":3000"))
}
