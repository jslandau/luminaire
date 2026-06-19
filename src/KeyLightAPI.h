#ifndef KEYLIGHTAPI_H
#define KEYLIGHTAPI_H

#include <QObject>
#include <QNetworkAccessManager>
#include <QNetworkReply>

class KeyLightAPI : public QObject
{
    Q_OBJECT

public:
    explicit KeyLightAPI(QObject *parent = nullptr);

    void setHost(const QString &ip, int port = 9123);
    QString host() const { return m_ip; }
    int port() const { return m_port; }

    void fetchState();
    void setPower(bool on);
    void setBrightness(int brightness);
    void setTemperature(int temperature);
    void setState(bool on, int brightness, int temperature);

    static int kelvinToApi(int kelvin);
    static int apiToKelvin(int apiValue);

    // Brightness constants
    static constexpr int MIN_BRIGHTNESS = 0;
    static constexpr int MAX_BRIGHTNESS = 100;

    // Temperature constants
    static constexpr int MIN_KELVIN = 2900;
    static constexpr int MAX_KELVIN = 7000;
    static constexpr int MIN_API_TEMP = 143;  // Corresponds to 7000K
    static constexpr int MAX_API_TEMP = 344;  // Corresponds to 2900K

signals:
    void stateReceived(bool on, int brightness, int temperature);
    void errorOccurred(const QString &error);
    // Emitted only when transitioning from not-yet-connected to connected,
    // not on every successful periodic refresh.
    void connectionSucceeded();

private slots:
    void onGetFinished(QNetworkReply *reply);
    void onPutFinished(QNetworkReply *reply);

private:
    QUrl lightsUrl() const;
    void sendPutRequest(const QByteArray &json);

    QNetworkAccessManager *m_manager;
    QString m_ip;
    int m_port = 9123;
    bool m_hasSuccessfulConnection = false;
};

#endif // KEYLIGHTAPI_H
