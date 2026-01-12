/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package tunnel

import (
	"fmt"

	"github.com/charmbracelet/lipgloss"
)

func (i tunnelItem) Description() string { return "" }
func (i tunnelItem) FilterValue() string { return i.config.Name }

func (i tunnelItem) Title() string {
	var status string
	var style lipgloss.Style

	status = i.config.Status
	if status == "" {
		status = "stopped"
		style = stoppedStyle
	} else {
		switch status {
		case "connected":
			style = runningStyle
		case "disconnected", "disconnected-ready":
			status = "disconnected"
			style = errorStyle
		case "reconnecting":
			style = pendingStyle
		default:
			style = runningStyle
		}
	}

	namePart := fmt.Sprintf(fmt.Sprintf("%%-%ds", i.maxNameLen), i.config.Name)
	portPart := fmt.Sprintf(fmt.Sprintf("%%%ds", i.maxPortLen), i.config.LocalPort)

	return fmt.Sprintf("%s     %s   %s", namePart, portPart, style.Render(status))
}
