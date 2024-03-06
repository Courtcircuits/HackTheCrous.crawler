package cmd

import (
	"fmt"
	"os"

	"github.com/Courtcircuits/HackTheCrous.crawler/controllers"
	"github.com/spf13/cobra"
)

var RootCmd = &cobra.Command{
	Use:   "htc",
	Short: "CLI utility to scrape Crous restaurants",
	Long:  `CLI utility to scrape Crous restaurants`,
}

func Execute() {
	controllers.Db = controllers.NewDatabase()
	if err := RootCmd.Execute(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}
