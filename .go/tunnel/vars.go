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
	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/lipgloss"
)

const (
	stateList sessionState = iota
	stateActions
	stateForm
)

var (
	docStyle      = lipgloss.NewStyle().Margin(0, 2)
	titleStyle    = lipgloss.NewStyle().Background(lipgloss.Color("62")).Foreground(lipgloss.Color("230")).Padding(0, 1)
	runningStyle  = lipgloss.NewStyle().Foreground(lipgloss.Color("42"))
	stoppedStyle  = lipgloss.NewStyle().Foreground(lipgloss.Color("241"))
	errorStyle    = lipgloss.NewStyle().Foreground(lipgloss.Color("196"))
	pendingStyle  = lipgloss.NewStyle().Foreground(lipgloss.Color("214"))
	selectedStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("205")).Bold(true)
	addKey        = key.NewBinding(key.WithKeys("a"), key.WithHelp("a", "add tunnel"))
	stopAllKey    = key.NewBinding(key.WithKeys("."), key.WithHelp(".", "stop all"))
)
