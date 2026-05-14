#include "MainWindow.h"
#include "Config.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QGroupBox>
#include <QIntValidator>
#include <QEvent>
#include <QMouseEvent>
#include <QCloseEvent>
#include <QApplication>
#include <QStyleHints>
#include <QPainter>
#include <QTimer>
#include <QDebug>

MainWindow::MainWindow(QWidget *parent)
    : QWidget(parent)
    , m_api(new KeyLightAPI(this))
    , m_refreshTimer(new QTimer(this))
{
    setWindowTitle("Luminaire");
    setWindowIcon(createLightbulbIcon(true));
    setMinimumWidth(300);

    auto *mainLayout = new QVBoxLayout(this);

    // Connection section
    auto *connGroup = new QGroupBox("Connection");
    auto *connLayout = new QHBoxLayout(connGroup);

    m_ipEdit = new QLineEdit;
    m_ipEdit->setPlaceholderText("192.168.1.100");
    m_ipEdit->setText(Config::loadIpAddress());

    m_connectBtn = new QPushButton("Connect");

    connLayout->addWidget(new QLabel("IP:"));
    connLayout->addWidget(m_ipEdit, 1);
    connLayout->addWidget(m_connectBtn);

    mainLayout->addWidget(connGroup);

    // Status
    m_statusLabel = new QLabel("Not connected");
    m_statusLabel->setStyleSheet("color: gray;");
    mainLayout->addWidget(m_statusLabel);

    // Power section
    auto *powerGroup = new QGroupBox("Power");
    auto *powerLayout = new QHBoxLayout(powerGroup);

    m_powerToggle = new QPushButton("OFF");
    m_powerToggle->setMinimumHeight(40);
    updatePowerButton(false);

    powerLayout->addWidget(m_powerToggle);

    mainLayout->addWidget(powerGroup);

    // Brightness section
    auto *brightnessGroup = new QGroupBox("Brightness");
    auto *brightnessLayout = new QVBoxLayout(brightnessGroup);

    m_brightnessStack = new QStackedWidget;
    m_brightnessLabel = new QLabel("0%");
    m_brightnessLabel->setAlignment(Qt::AlignCenter);
    m_brightnessLabel->setCursor(Qt::PointingHandCursor);
    m_brightnessLabel->installEventFilter(this);

    m_brightnessEdit = new QLineEdit;
    m_brightnessEdit->setAlignment(Qt::AlignCenter);
    m_brightnessEdit->setValidator(new QIntValidator(KeyLightAPI::MIN_BRIGHTNESS, KeyLightAPI::MAX_BRIGHTNESS, this));

    m_brightnessStack->addWidget(m_brightnessLabel);
    m_brightnessStack->addWidget(m_brightnessEdit);

    m_brightnessSlider = new QSlider(Qt::Horizontal);
    m_brightnessSlider->setRange(KeyLightAPI::MIN_BRIGHTNESS, KeyLightAPI::MAX_BRIGHTNESS);
    m_brightnessSlider->setValue((KeyLightAPI::MIN_BRIGHTNESS + KeyLightAPI::MAX_BRIGHTNESS) / 2);

    brightnessLayout->addWidget(m_brightnessStack);
    brightnessLayout->addWidget(m_brightnessSlider);

    mainLayout->addWidget(brightnessGroup);

    // Temperature section
    auto *tempGroup = new QGroupBox("Temperature");
    auto *tempLayout = new QVBoxLayout(tempGroup);

    m_temperatureStack = new QStackedWidget;
    m_temperatureLabel = new QLabel("4500K");
    m_temperatureLabel->setAlignment(Qt::AlignCenter);
    m_temperatureLabel->setCursor(Qt::PointingHandCursor);
    m_temperatureLabel->installEventFilter(this);

    m_temperatureEdit = new QLineEdit;
    m_temperatureEdit->setAlignment(Qt::AlignCenter);
    m_temperatureEdit->setValidator(new QIntValidator(KeyLightAPI::MIN_KELVIN, KeyLightAPI::MAX_KELVIN, this));

    m_temperatureStack->addWidget(m_temperatureLabel);
    m_temperatureStack->addWidget(m_temperatureEdit);

    m_temperatureSlider = new QSlider(Qt::Horizontal);
    m_temperatureSlider->setRange(KeyLightAPI::MIN_KELVIN, KeyLightAPI::MAX_KELVIN);
    m_temperatureSlider->setValue((KeyLightAPI::MIN_KELVIN + KeyLightAPI::MAX_KELVIN) / 2);

    auto *tempRangeLayout = new QHBoxLayout;
    tempRangeLayout->addWidget(new QLabel(QString("%1K").arg(KeyLightAPI::MIN_KELVIN)));
    tempRangeLayout->addStretch();
    tempRangeLayout->addWidget(new QLabel(QString("%1K").arg(KeyLightAPI::MAX_KELVIN)));

    tempLayout->addWidget(m_temperatureStack);
    tempLayout->addWidget(m_temperatureSlider);
    tempLayout->addLayout(tempRangeLayout);

    mainLayout->addWidget(tempGroup);

    mainLayout->addStretch();

    // Initially disable controls
    setControlsEnabled(false);

    // Connect signals
    connect(m_connectBtn, &QPushButton::clicked, this, &MainWindow::onConnectClicked);
    connect(m_ipEdit, &QLineEdit::returnPressed, this, &MainWindow::onConnectClicked);
    connect(m_powerToggle, &QPushButton::clicked, this, &MainWindow::onPowerToggled);

    // Slider: update label on any change, but only send API on release
    connect(m_brightnessSlider, &QSlider::valueChanged, this, &MainWindow::onBrightnessSliderMoved);
    connect(m_brightnessSlider, &QSlider::sliderReleased, this, &MainWindow::onBrightnessSliderReleased);
    connect(m_temperatureSlider, &QSlider::valueChanged, this, &MainWindow::onTemperatureSliderMoved);
    connect(m_temperatureSlider, &QSlider::sliderReleased, this, &MainWindow::onTemperatureSliderReleased);

    // Edit fields
    connect(m_brightnessEdit, &QLineEdit::editingFinished, this, &MainWindow::onBrightnessEditFinished);
    connect(m_temperatureEdit, &QLineEdit::editingFinished, this, &MainWindow::onTemperatureEditFinished);

    connect(m_api, &KeyLightAPI::stateReceived, this, &MainWindow::onStateReceived);
    connect(m_api, &KeyLightAPI::connectionSucceeded, this, &MainWindow::onConnectionSucceeded);
    connect(m_api, &KeyLightAPI::errorOccurred, this, &MainWindow::onError);

    // Periodic refresh to stay in sync with external changes
    m_refreshTimer->setInterval(5000);
    connect(m_refreshTimer, &QTimer::timeout, m_api, &KeyLightAPI::fetchState);

    updateBrightnessDisplay(m_brightnessSlider->value());
    updateTemperatureDisplay(m_temperatureSlider->value());

    setupTrayIcon();

    // Auto-connect if we have a saved IP
    QString savedIp = Config::loadIpAddress();
    if (!savedIp.isEmpty()) {
        QTimer::singleShot(0, this, &MainWindow::onConnectClicked);
    }
}

void MainWindow::showWindow()
{
    // Clear minimized state if present
    if (isMinimized()) {
        setWindowState(windowState() & ~Qt::WindowMinimized);
    }

    show();

#ifdef Q_OS_LINUX
    // On Wayland (especially KDE Plasma), window activation is restricted.
    // We need to use a combination of techniques to bring window to front.
    setWindowState(Qt::WindowActive);
    raise();
    activateWindow();

    // Force focus - helps on some platforms
    setFocus();
#endif

    updateShowHideAction();
}

void MainWindow::hideWindow()
{
    hide();
    updateShowHideAction();
}

void MainWindow::updateShowHideAction()
{
    if (m_trayShowHideAction) {
        m_trayShowHideAction->setText(isVisible() && !isMinimized() ? "Hide Window" : "Show Window");
    }
}

void MainWindow::toggleWindow()
{
    if (isVisible() && !isMinimized()) {
        hideWindow();
    } else {
        showWindow();
    }
}

void MainWindow::setupTrayIcon()
{
    m_trayIcon = new QSystemTrayIcon(this);
    m_trayIcon->setToolTip("Key Light Control");

    // Create tray menu
    m_trayMenu = new QMenu(this);

    m_trayPowerOnAction = m_trayMenu->addAction("Light On");
    m_trayPowerOffAction = m_trayMenu->addAction("Light Off");
    m_trayMenu->addSeparator();
    m_trayShowHideAction = m_trayMenu->addAction("Show Window");
    m_trayMenu->addSeparator();
#ifdef Q_OS_MACOS
    QAction *exitAction = m_trayMenu->addAction("Quit");
#else
    QAction *exitAction = m_trayMenu->addAction("Exit");
#endif

#ifndef Q_OS_MACOS
    m_trayIcon->setContextMenu(m_trayMenu);
#endif

    // Connect tray actions
    connect(m_trayPowerOnAction, &QAction::triggered, this, [this]() { m_api->setPower(true); });
    connect(m_trayPowerOffAction, &QAction::triggered, this, [this]() { m_api->setPower(false); });
    connect(m_trayShowHideAction, &QAction::triggered, this, [this]() {
        qDebug() << "Show/Hide Window clicked - isVisible:" << isVisible() << "isMinimized:" << isMinimized();
        toggleWindow();
    });
    connect(exitAction, &QAction::triggered, qApp, &QApplication::quit);

    connect(m_trayIcon, &QSystemTrayIcon::activated, this, &MainWindow::onTrayActivated);

#ifdef Q_OS_MACOS
    connect(qApp->styleHints(), &QStyleHints::colorSchemeChanged, this, [this](Qt::ColorScheme) {
        updateTrayIcon(m_lightOn);
    });
#endif

    updateTrayIcon(false);
    updateTrayActions();
    updateShowHideAction();
    m_trayIcon->show();
}

QIcon MainWindow::createLightbulbIcon(bool on)
{
    QPixmap pixmap(32, 32);
    pixmap.fill(Qt::transparent);

    QPainter painter(&pixmap);
    painter.setRenderHint(QPainter::Antialiasing);

    // Bulb color
#ifdef Q_OS_MACOS
    const bool darkMode = QGuiApplication::styleHints()->colorScheme() == Qt::ColorScheme::Dark;
    QColor bulbColor = on ? QColor(255, 220, 80) : (darkMode ? QColor(200, 200, 200) : QColor(128, 128, 128));
    QColor outlineColor = on ? QColor(200, 160, 40) : (darkMode ? QColor(150, 150, 150) : QColor(80, 80, 80));
#else
    QColor bulbColor = on ? QColor(255, 220, 80) : QColor(128, 128, 128);
    QColor outlineColor = on ? QColor(200, 160, 40) : QColor(80, 80, 80);
#endif

    // Draw glow if on
    if (on) {
        QRadialGradient glow(16, 12, 14);
        glow.setColorAt(0, QColor(255, 240, 150, 180));
        glow.setColorAt(1, QColor(255, 240, 150, 0));
        painter.setBrush(glow);
        painter.setPen(Qt::NoPen);
        painter.drawEllipse(2, 0, 28, 24);
    }

    // Draw bulb (oval)
    painter.setBrush(bulbColor);
    painter.setPen(QPen(outlineColor, 1.5));
    painter.drawEllipse(6, 2, 20, 18);

    // Draw base/screw part
    QColor baseColor = on ? QColor(180, 180, 180) : QColor(100, 100, 100);
    painter.setBrush(baseColor);
    painter.setPen(QPen(baseColor.darker(120), 1));

    // Trapezoid base
    QPolygonF base;
    base << QPointF(10, 19) << QPointF(22, 19) << QPointF(20, 24) << QPointF(12, 24);
    painter.drawPolygon(base);

    // Screw threads
    painter.drawRect(12, 24, 8, 2);
    painter.drawRect(13, 26, 6, 2);
    painter.drawRect(14, 28, 4, 2);

    return QIcon(pixmap);
}

void MainWindow::updateTrayIcon(bool lightOn)
{
    m_trayIcon->setIcon(createLightbulbIcon(lightOn));
    m_trayIcon->setToolTip(QString("Key Light Control - %1").arg(lightOn ? "On" : "Off"));
}

void MainWindow::updateTrayActions()
{
    m_trayPowerOnAction->setEnabled(m_connected);
    m_trayPowerOffAction->setEnabled(m_connected);
}

void MainWindow::onTrayActivated(QSystemTrayIcon::ActivationReason reason)
{
    qDebug() << "Tray activated with reason:" << reason;

#ifdef Q_OS_MACOS
    if (reason == QSystemTrayIcon::Context) {
        if (m_trayMenu->isVisible()) {
            m_trayMenu->hide();
        } else {
            m_trayMenu->popup(QCursor::pos());
        }
        return;
    }
#endif

    if (reason == QSystemTrayIcon::Trigger) {
        // Single-click: toggle light power
        qDebug() << "Single-click detected - toggling light";
        if (m_connected) {
            m_api->setPower(!m_lightOn);
        }
    } else if (reason == QSystemTrayIcon::DoubleClick) {
        // Double-click: show/hide window (Linux only — on macOS Qt can't distinguish
        // left vs right double-click, so this would fire on rapid right-clicks too)
#ifndef Q_OS_MACOS
        qDebug() << "Double-click detected - toggling window";
        toggleWindow();
#endif
#ifndef Q_OS_MACOS
    } else if (reason == QSystemTrayIcon::MiddleClick) {
        // Middle-click: show/hide window (more reliable on Linux)
        qDebug() << "Middle-click detected - toggling window";
        toggleWindow();
#endif
    }
}

void MainWindow::closeEvent(QCloseEvent *event)
{
    if (m_trayIcon->isVisible()) {
        hide();
        updateShowHideAction();
        event->ignore();
    } else {
        event->accept();
    }
}

void MainWindow::changeEvent(QEvent *event)
{
#ifdef Q_OS_MACOS
    if (event->type() == QEvent::PaletteChange) {
        updateTrayIcon(m_lightOn);
    }
#endif
    QWidget::changeEvent(event);
}

bool MainWindow::eventFilter(QObject *obj, QEvent *event)
{
    if (event->type() == QEvent::MouseButtonDblClick) {
        if (obj == m_brightnessLabel) {
            startBrightnessEdit();
            return true;
        } else if (obj == m_temperatureLabel) {
            startTemperatureEdit();
            return true;
        }
    }
    return QWidget::eventFilter(obj, event);
}

void MainWindow::startBrightnessEdit()
{
    m_brightnessEdit->setText(QString::number(m_brightnessSlider->value()));
    m_brightnessStack->setCurrentWidget(m_brightnessEdit);
    m_brightnessEdit->setFocus();
    m_brightnessEdit->selectAll();
}

void MainWindow::startTemperatureEdit()
{
    m_temperatureEdit->setText(QString::number(m_temperatureSlider->value()));
    m_temperatureStack->setCurrentWidget(m_temperatureEdit);
    m_temperatureEdit->setFocus();
    m_temperatureEdit->selectAll();
}

void MainWindow::onBrightnessEditFinished()
{
    int value = m_brightnessEdit->text().toInt();
    value = qBound(KeyLightAPI::MIN_BRIGHTNESS, value, KeyLightAPI::MAX_BRIGHTNESS);
    m_brightnessSlider->setValue(value);
    m_brightnessStack->setCurrentWidget(m_brightnessLabel);
    if (m_connected) {
        m_api->setBrightness(value);
        Config::saveBrightness(value);
    }
}

void MainWindow::onTemperatureEditFinished()
{
    int value = m_temperatureEdit->text().toInt();
    value = qBound(KeyLightAPI::MIN_KELVIN, value, KeyLightAPI::MAX_KELVIN);
    m_temperatureSlider->setValue(value);
    m_temperatureStack->setCurrentWidget(m_temperatureLabel);
    if (m_connected) {
        m_api->setTemperature(value);
        Config::saveTemperature(value);
    }
}

void MainWindow::setControlsEnabled(bool enabled)
{
    m_powerToggle->setEnabled(enabled);
    m_brightnessSlider->setEnabled(enabled);
    m_temperatureSlider->setEnabled(enabled);
    m_brightnessLabel->setEnabled(enabled);
    m_temperatureLabel->setEnabled(enabled);
}

void MainWindow::updateBrightnessDisplay(int value)
{
    m_brightnessLabel->setText(QString("%1%").arg(value));
}

void MainWindow::updateTemperatureDisplay(int value)
{
    m_temperatureLabel->setText(QString("%1K").arg(value));
}

void MainWindow::updatePowerButton(bool on)
{
    m_lightOn = on;
    m_powerToggle->setText(on ? "ON" : "OFF");
    m_powerToggle->setStyleSheet(on
        ? "background-color: #4CAF50; color: white; font-weight: bold;"
        : "background-color: #f44336; color: white; font-weight: bold;");
}

void MainWindow::onConnectClicked()
{
    if (m_connected) {
        // Disconnect
        m_connected = false;
        m_consecutiveErrors = 0;
        m_refreshTimer->stop();
        m_statusLabel->setText("Disconnected");
        m_statusLabel->setStyleSheet("color: gray;");
        m_connectBtn->setText("Connect");
        m_ipEdit->setEnabled(true);
        setControlsEnabled(false);
        updateTrayActions();
        updateTrayIcon(false);
        return;
    }

    // Connect
    QString ip = m_ipEdit->text().trimmed();
    if (ip.isEmpty()) {
        m_statusLabel->setText("Please enter an IP address");
        m_statusLabel->setStyleSheet("color: red;");
        return;
    }

    Config::saveIpAddress(ip);
    m_api->setHost(ip);
    m_consecutiveErrors = 0;

    m_statusLabel->setText("Connecting...");
    m_statusLabel->setStyleSheet("color: orange;");
    m_connectBtn->setEnabled(false);

    m_api->fetchState();
}

void MainWindow::onConnectionSucceeded()
{
    m_connected = true;
    m_statusLabel->setText(QString("Connected to %1").arg(m_api->host()));
    m_statusLabel->setStyleSheet("color: green;");
    m_connectBtn->setText("Disconnect");
    m_connectBtn->setEnabled(true);
    m_ipEdit->setEnabled(false);
    setControlsEnabled(true);
    updateTrayActions();
    m_refreshTimer->start();

    // Restore saved brightness/temperature if available
    int savedBrightness = Config::loadBrightness();
    int savedTemperature = Config::loadTemperature();
    if (savedBrightness >= 0 || savedTemperature >= 0) {
        if (savedBrightness >= 0) {
            m_brightnessSlider->setValue(savedBrightness);
            m_api->setBrightness(savedBrightness);
        }
        if (savedTemperature >= 0) {
            m_temperatureSlider->setValue(savedTemperature);
            m_api->setTemperature(savedTemperature);
        }
    }
}

void MainWindow::onError(const QString &error)
{
    m_consecutiveErrors++;

    // After MAX_CONSECUTIVE_ERRORS, treat as fully disconnected
    if (m_consecutiveErrors >= MAX_CONSECUTIVE_ERRORS) {
        m_connected = false;
        m_statusLabel->setText("Connection lost: " + error);
        m_statusLabel->setStyleSheet("color: red;");
        m_connectBtn->setText("Connect");
        m_connectBtn->setEnabled(true);
        m_ipEdit->setEnabled(true);
        setControlsEnabled(false);
        updateTrayActions();
        updateTrayIcon(false);
        m_refreshTimer->stop();
    } else {
        // Show temporary error but stay connected for retry
        m_statusLabel->setText(QString("Error (%1/%2): %3")
            .arg(m_consecutiveErrors)
            .arg(MAX_CONSECUTIVE_ERRORS)
            .arg(error));
        m_statusLabel->setStyleSheet("color: orange;");
    }
}

void MainWindow::onStateReceived(bool on, int brightness, int temperature)
{
    // Reset error counter on successful response
    if (m_consecutiveErrors > 0) {
        m_consecutiveErrors = 0;
        if (m_connected) {
            m_statusLabel->setText(QString("Connected to %1").arg(m_api->host()));
            m_statusLabel->setStyleSheet("color: green;");
        }
    }

    // Always update tray icon and power button
    updateTrayIcon(on);
    updatePowerButton(on);

    // Only update sliders if not currently dragging
    if (m_brightnessSlider->isSliderDown() || m_temperatureSlider->isSliderDown()) {
        return;
    }

    m_brightnessSlider->setValue(brightness);
    m_temperatureSlider->setValue(temperature);
}

void MainWindow::onPowerToggled()
{
    m_api->setPower(!m_lightOn);
}

void MainWindow::onBrightnessSliderMoved(int value)
{
    updateBrightnessDisplay(value);
}

void MainWindow::onBrightnessSliderReleased()
{
    if (m_connected) {
        int value = m_brightnessSlider->value();
        m_api->setBrightness(value);
        Config::saveBrightness(value);
    }
}

void MainWindow::onTemperatureSliderMoved(int value)
{
    updateTemperatureDisplay(value);
}

void MainWindow::onTemperatureSliderReleased()
{
    if (m_connected) {
        int value = m_temperatureSlider->value();
        m_api->setTemperature(value);
        Config::saveTemperature(value);
    }
}
