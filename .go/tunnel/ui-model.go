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
	"os"
	"os/exec"
	"strings"

	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

func (m stuItemModel) Init() tea.Cmd {
	return tick()
}

func (m *stuItemModel) ResizeList() {
	h, v := docStyle.GetFrameSize()
	if m.list.Help.ShowAll {
		m.list.SetSize(m.lastWidth-h, m.lastHeight-v)
	} else {
		m.list.SetSize(m.lastWidth-h, m.lastHeight-v-2)
	}
}

func (m stuItemModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		key := msg.String()

		switch m.state {
		case stateList:
			if key == "enter" {
				if m.list.FilterState() == list.Filtering {
					m.list.Update(msg)
				}

				if item, ok := m.list.SelectedItem().(stuTunnelItem); ok {
					m.selected = item
					m.state = stateActions
					m.actionChoice = 0
					m.updateActions()
					return m, nil
				}
			}

			if key == "a" && m.list.FilterState() != list.Filtering {
				m.state = stateForm
				return m, tea.ExecProcess(exec.Command("clear"), func(err error) tea.Msg {
					addTunnelLogic()
					return stuFormFinishedMsg{}
				})
			}

			if key == "?" {
				m.list.SetShowHelp(!m.list.ShowHelp())
				m.list.Help.ShowAll = m.list.ShowHelp()
				m.ResizeList()
				return m, nil
			}

			if key == "esc" && m.list.Help.ShowAll {
				m.list.SetShowHelp(false)
				m.list.Help.ShowAll = false
				m.ResizeList()
				return m, nil
			}

			if key == "." && m.list.FilterState() != list.Filtering {
				stopAllTunnelsLogic()
				return m, nil
			}

			if key == "ctrl+c" || (key == "q" && m.list.FilterState() != list.Filtering) {
				m.quitting = true
				return m, tea.Quit
			}

		case stateActions:
			switch key {
			case "ctrl+c", "esc", "q":
				m.state = stateList
				return m, nil

			case "up", "k":
				if m.actionChoice > 0 {
					m.actionChoice--
				}

			case "down", "j":
				if m.actionChoice < len(m.actions)-1 {
					m.actionChoice++
				}

			case "enter":
				action := m.actions[m.actionChoice]
				return m.handleAction(action)
			}
		}

	case tea.WindowSizeMsg:
		m.lastWidth = msg.Width
		m.lastHeight = msg.Height
		m.ResizeList()
		return m, nil

	case stuFormFinishedMsg:
		m.state = stateList
		return m, nil

	case tickMsg:
		cfg, err := loadConfig()
		if err == nil {
			filteredTunnels := filterTunnels(cfg.Tunnels, m.sshMode)
			maxN, maxP := calculateMaxLengths(filteredTunnels)

			if m.state == stateActions {
				if t, found := cfg.getTunnel(m.selected.config.Name); found {
					isRunning := isTunnelRunning(t.PID)
					displayStatus := syncStatus(t.Name, isRunning, t.Status, t.LocalPort)

					t.Status = displayStatus
					m.selected = stuTunnelItem{config: t, running: isRunning, sshMode: m.sshMode, maxNameLen: maxN, maxPortLen: maxP}
					m.updateActions()
				}
			}

			if m.state == stateList {
				var selectedName string
				if item, ok := m.list.SelectedItem().(stuTunnelItem); ok {
					selectedName = item.config.Name
				}

				var currentItems []list.Item
				for _, t := range filteredTunnels {
					isRunning := isTunnelRunning(t.PID)
					displayStatus := syncStatus(t.Name, isRunning, t.Status, t.LocalPort)

					t.Status = displayStatus
					item := stuTunnelItem{config: t, running: isRunning, sshMode: m.sshMode, maxNameLen: maxN, maxPortLen: maxP}
					currentItems = append(currentItems, item)
				}
				m.list.SetItems(currentItems)

				if selectedName != "" {
					for i, item := range m.list.Items() {
						if ti, ok := item.(stuTunnelItem); ok && ti.config.Name == selectedName {
							m.list.Select(i)
							break
						}
					}
				}
			}
		}

		return m, tick()
	}

	if m.state == stateList {
		var cmd tea.Cmd
		m.list, cmd = m.list.Update(msg)
		return m, cmd
	}

	return m, nil
}

func (m stuItemModel) View() string {
	if m.quitting || m.state == stateForm {
		return ""
	}

	if m.state == stateActions {
		statusLine := stoppedStyle.Render("stopped")
		status := m.selected.config.Status
		if status == "disconnected-ready" {
			status = "disconnected"
		}

		if status != "" {
			switch status {
			case "connected":
				statusLine = runningStyle.Render("connected")
			case "disconnected":
				statusLine = errorStyle.Render("disconnected")
			case "reconnecting":
				statusLine = pendingStyle.Render("reconnecting")
			default:
				statusLine = runningStyle.Render(status)
			}
		}

		s := ""
		if m.sshMode {
			s += "\n"
		} else {
			s += fmt.Sprintf("\n  --- Tunnel: %s %s ---\n\n", m.selected.config.Name, statusLine)
		}

		for i, action := range m.actions {
			cursor := "  "
			style := lipgloss.NewStyle()
			if m.actionChoice == i {
				cursor = "> "
				style = selectedStyle
			}
			s += fmt.Sprintf("  %s%s\n", cursor, style.Render(action))
		}

		s += "\n  (esc to go back)\n"
		return docStyle.Render(s)
	}

	if m.list.Help.ShowAll {
		return docStyle.Render(m.list.View())
	}

	return docStyle.Render(m.list.View() + "\n\n  " + lipgloss.NewStyle().Foreground(lipgloss.Color("241")).Render("↑↓ • . stop all • / filter • q quit • ? more"))
}

func (m *stuItemModel) updateActions() {
	m.actions = make([]string, 0)
	if !m.sshMode && strings.Contains(m.selected.config.Actions, "tunnel") && m.selected.config.LocalPort != "" {
		if m.selected.running {
			m.actions = append(m.actions, "Stop")
		} else {
			m.actions = append(m.actions, "Start")
		}
	}

	if m.sshMode && strings.Contains(m.selected.config.Actions, "ssh") {
		m.actions = append(m.actions, "Access")
	}

	m.actions = append(m.actions, "Update", "Delete")

	if m.actionChoice >= len(m.actions) {
		m.actionChoice = 0
	}
}

func (m stuItemModel) handleAction(action string) (tea.Model, tea.Cmd) {
	name := m.selected.config.Name

	switch action {
	case "Access":
		cmd := buildSshCmd(m.selected.config)
		cmd.Stdin = os.Stdin
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr

		m.pendingCmd = cmd
		return m, tea.Quit

	case "Start":
		startTunnelLogic(name)
		m.state = stateList
		return m, nil

	case "Stop":
		stopTunnelLogic(name)
		m.state = stateList
		return m, nil

	case "Delete":
		m.state = stateForm
		return m, tea.ExecProcess(exec.Command("clear"), func(err error) tea.Msg {
			deleteTunnelLogic(name)
			return stuFormFinishedMsg{}
		})

	case "Update":
		m.state = stateForm
		return m, tea.ExecProcess(exec.Command("clear"), func(err error) tea.Msg {
			updateTunnelLogic(name)
			return stuFormFinishedMsg{}
		})
	}
	return m, nil
}
