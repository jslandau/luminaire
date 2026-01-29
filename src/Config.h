#ifndef CONFIG_H
#define CONFIG_H

#include <QString>

class Config
{
public:
    static QString loadIpAddress();
    static void saveIpAddress(const QString &ip);

    static int loadBrightness();
    static void saveBrightness(int brightness);

    static int loadTemperature();
    static void saveTemperature(int temperature);
};

#endif // CONFIG_H
