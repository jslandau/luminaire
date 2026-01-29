#include "KeyLightAPI.h"
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>
#include <QNetworkRequest>

KeyLightAPI::KeyLightAPI(QObject *parent)
    : QObject(parent)
    , m_manager(new QNetworkAccessManager(this))
{
}

void KeyLightAPI::setHost(const QString &ip, int port)
{
    m_ip = ip;
    m_port = port;
}

QUrl KeyLightAPI::lightsUrl() const
{
    return QUrl(QString("http://%1:%2/elgato/lights").arg(m_ip).arg(m_port));
}

void KeyLightAPI::fetchState()
{
    if (m_ip.isEmpty()) {
        emit errorOccurred("No IP address configured");
        return;
    }

    QNetworkRequest request(lightsUrl());
    QNetworkReply *reply = m_manager->get(request);
    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        onGetFinished(reply);
    });
}

void KeyLightAPI::onGetFinished(QNetworkReply *reply)
{
    reply->deleteLater();

    if (reply->error() != QNetworkReply::NoError) {
        emit errorOccurred(reply->errorString());
        return;
    }

    QByteArray data = reply->readAll();
    QJsonDocument doc = QJsonDocument::fromJson(data);
    if (doc.isNull()) {
        emit errorOccurred("Invalid JSON response");
        return;
    }

    QJsonObject root = doc.object();
    QJsonArray lights = root["lights"].toArray();
    if (lights.isEmpty()) {
        emit errorOccurred("No lights found in response");
        return;
    }

    QJsonObject light = lights[0].toObject();
    bool on = light["on"].toInt() == 1;
    int brightness = light["brightness"].toInt();
    int temperature = light["temperature"].toInt();

    emit connectionSucceeded();
    emit stateReceived(on, brightness, apiToKelvin(temperature));
}

void KeyLightAPI::sendPutRequest(const QByteArray &json)
{
    if (m_ip.isEmpty()) {
        emit errorOccurred("No IP address configured");
        return;
    }

    QNetworkRequest request(lightsUrl());
    request.setHeader(QNetworkRequest::ContentTypeHeader, "application/json");
    QNetworkReply *reply = m_manager->put(request, json);
    connect(reply, &QNetworkReply::finished, this, [this, reply]() {
        onPutFinished(reply);
    });
}

void KeyLightAPI::onPutFinished(QNetworkReply *reply)
{
    reply->deleteLater();

    if (reply->error() != QNetworkReply::NoError) {
        emit errorOccurred(reply->errorString());
        return;
    }

    // Parse response to update UI with actual values
    QByteArray data = reply->readAll();
    QJsonDocument doc = QJsonDocument::fromJson(data);
    if (!doc.isNull()) {
        QJsonObject root = doc.object();
        QJsonArray lights = root["lights"].toArray();
        if (!lights.isEmpty()) {
            QJsonObject light = lights[0].toObject();
            bool on = light["on"].toInt() == 1;
            int brightness = light["brightness"].toInt();
            int temperature = light["temperature"].toInt();
            emit stateReceived(on, brightness, apiToKelvin(temperature));
        }
    }
}

void KeyLightAPI::setPower(bool on)
{
    QJsonObject light;
    light["on"] = on ? 1 : 0;

    QJsonArray lights;
    lights.append(light);

    QJsonObject root;
    root["numberOfLights"] = 1;
    root["lights"] = lights;

    sendPutRequest(QJsonDocument(root).toJson(QJsonDocument::Compact));
}

void KeyLightAPI::setBrightness(int brightness)
{
    brightness = qBound(MIN_BRIGHTNESS, brightness, MAX_BRIGHTNESS);

    QJsonObject light;
    light["brightness"] = brightness;

    QJsonArray lights;
    lights.append(light);

    QJsonObject root;
    root["numberOfLights"] = 1;
    root["lights"] = lights;

    sendPutRequest(QJsonDocument(root).toJson(QJsonDocument::Compact));
}

void KeyLightAPI::setTemperature(int kelvin)
{
    QJsonObject light;
    light["temperature"] = kelvinToApi(kelvin);

    QJsonArray lights;
    lights.append(light);

    QJsonObject root;
    root["numberOfLights"] = 1;
    root["lights"] = lights;

    sendPutRequest(QJsonDocument(root).toJson(QJsonDocument::Compact));
}

void KeyLightAPI::setState(bool on, int brightness, int kelvin)
{
    brightness = qBound(MIN_BRIGHTNESS, brightness, MAX_BRIGHTNESS);

    QJsonObject light;
    light["on"] = on ? 1 : 0;
    light["brightness"] = brightness;
    light["temperature"] = kelvinToApi(kelvin);

    QJsonArray lights;
    lights.append(light);

    QJsonObject root;
    root["numberOfLights"] = 1;
    root["lights"] = lights;

    sendPutRequest(QJsonDocument(root).toJson(QJsonDocument::Compact));
}

int KeyLightAPI::kelvinToApi(int kelvin)
{
    // API uses inverse scale: MIN_API_TEMP(143) = MAX_KELVIN(7000K), MAX_API_TEMP(344) = MIN_KELVIN(2900K)
    // Linear interpolation: api = MAX_API_TEMP - (kelvin - MIN_KELVIN) * (MAX_API_TEMP - MIN_API_TEMP) / (MAX_KELVIN - MIN_KELVIN)
    kelvin = qBound(MIN_KELVIN, kelvin, MAX_KELVIN);
    return MAX_API_TEMP - (kelvin - MIN_KELVIN) * (MAX_API_TEMP - MIN_API_TEMP) / (MAX_KELVIN - MIN_KELVIN);
}

int KeyLightAPI::apiToKelvin(int apiValue)
{
    // Inverse of kelvinToApi
    apiValue = qBound(MIN_API_TEMP, apiValue, MAX_API_TEMP);
    return MIN_KELVIN + (MAX_API_TEMP - apiValue) * (MAX_KELVIN - MIN_KELVIN) / (MAX_API_TEMP - MIN_API_TEMP);
}
