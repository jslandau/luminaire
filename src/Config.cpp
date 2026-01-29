#include "Config.h"
#include <QSettings>
#include <QCoreApplication>

// Note: Creating multiple QSettings instances is the recommended Qt pattern.
// Qt handles internal caching, so this is efficient. QSettings automatically
// uses organization/application name from QCoreApplication (set in main.cpp).

QString Config::loadIpAddress()
{
    QSettings settings(QCoreApplication::organizationName(), QCoreApplication::applicationName());
    return settings.value("ip_address", "").toString();
}

void Config::saveIpAddress(const QString &ip)
{
    QSettings settings(QCoreApplication::organizationName(), QCoreApplication::applicationName());
    settings.setValue("ip_address", ip);
    settings.sync();  // Ensure immediate write
}

int Config::loadBrightness()
{
    QSettings settings(QCoreApplication::organizationName(), QCoreApplication::applicationName());
    return settings.value("brightness", -1).toInt();
}

void Config::saveBrightness(int brightness)
{
    QSettings settings(QCoreApplication::organizationName(), QCoreApplication::applicationName());
    settings.setValue("brightness", brightness);
    settings.sync();  // Ensure immediate write
}

int Config::loadTemperature()
{
    QSettings settings(QCoreApplication::organizationName(), QCoreApplication::applicationName());
    return settings.value("temperature", -1).toInt();
}

void Config::saveTemperature(int temperature)
{
    QSettings settings(QCoreApplication::organizationName(), QCoreApplication::applicationName());
    settings.setValue("temperature", temperature);
    settings.sync();  // Ensure immediate write
}
