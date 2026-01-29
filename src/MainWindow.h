#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QWidget>
#include <QLineEdit>
#include <QPushButton>
#include <QSlider>
#include <QLabel>
#include <QStackedWidget>
#include <QSystemTrayIcon>
#include <QMenu>
#include <QTimer>
#include "KeyLightAPI.h"

class MainWindow : public QWidget
{
    Q_OBJECT

public:
    explicit MainWindow(QWidget *parent = nullptr);

protected:
    bool eventFilter(QObject *obj, QEvent *event) override;
    void closeEvent(QCloseEvent *event) override;

private slots:
    void onTrayActivated(QSystemTrayIcon::ActivationReason reason);
    void onConnectClicked();
    void onPowerToggled();
    void onBrightnessSliderMoved(int value);
    void onBrightnessSliderReleased();
    void onTemperatureSliderMoved(int value);
    void onTemperatureSliderReleased();
    void onBrightnessEditFinished();
    void onTemperatureEditFinished();
    void onStateReceived(bool on, int brightness, int temperature);
    void onConnectionSucceeded();
    void onError(const QString &error);

private:
    void setControlsEnabled(bool enabled);
    void updatePowerButton(bool on);
    void updateBrightnessDisplay(int value);
    void updateTemperatureDisplay(int value);
    void startBrightnessEdit();
    void startTemperatureEdit();
    void setupTrayIcon();
    void updateTrayIcon(bool lightOn);
    void updateTrayActions();
    void updateShowHideAction();
    void showWindow();
    void hideWindow();
    void toggleWindow();
    QIcon createLightbulbIcon(bool on);

    KeyLightAPI *m_api;
    QTimer *m_refreshTimer;
    int m_consecutiveErrors = 0;
    static constexpr int MAX_CONSECUTIVE_ERRORS = 3;

    QSystemTrayIcon *m_trayIcon;
    QMenu *m_trayMenu;
    QAction *m_trayPowerOnAction;
    QAction *m_trayPowerOffAction;
    QAction *m_trayShowHideAction;

    QLineEdit *m_ipEdit;
    QPushButton *m_connectBtn;
    QLabel *m_statusLabel;

    QPushButton *m_powerToggle;
    bool m_lightOn = false;

    QSlider *m_brightnessSlider;
    QStackedWidget *m_brightnessStack;
    QLabel *m_brightnessLabel;
    QLineEdit *m_brightnessEdit;

    QSlider *m_temperatureSlider;
    QStackedWidget *m_temperatureStack;
    QLabel *m_temperatureLabel;
    QLineEdit *m_temperatureEdit;

    bool m_connected = false;
};

#endif // MAINWINDOW_H
