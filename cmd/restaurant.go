package cmd

import (
	"github.com/Courtcircuits/HackTheCrous.crawler/controllers"
	"github.com/spf13/cobra"
)

var rc *controllers.RestaurantController

func init() {
	rc = controllers.NewRestaurantController()
	RootCmd.AddCommand(restaurantCmd)
}

var restaurantCmd = &cobra.Command{
	Use:   "restaurant",
	Short: "Scrape restaurant urls",
	Long:  `Scrape restaurant urls`,
	Run: func(cmd *cobra.Command, args []string) {
		controllers.ScrapeRestaurants("https://www.crous-montpellier.fr/se-restaurer/ou-manger/")
	},
}
