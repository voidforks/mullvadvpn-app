#include "stdafx.h"
#include "logger.h"
#include <libcommon/error.h>
#include <libcommon/string.h>
#include <sstream>
#include <iomanip>

AnsiFileLogSink::AnsiFileLogSink(const std::wstring &file, bool append, bool flush)
	: m_flush(flush)
{
	const DWORD creationDisposition = (append ? OPEN_ALWAYS : CREATE_ALWAYS);

	m_logfile = CreateFileW(file.c_str(), GENERIC_READ | GENERIC_WRITE, FILE_SHARE_READ, nullptr,
		creationDisposition, FILE_ATTRIBUTE_NORMAL, nullptr);

	THROW_GLE_IF(INVALID_HANDLE_VALUE, m_logfile, "Open/create log file");

	if (append && ERROR_ALREADY_EXISTS == GetLastError())
	{
		LARGE_INTEGER offset = { 0 };

		const auto seekStatus = SetFilePointerEx(m_logfile, offset, nullptr, FILE_END);

		THROW_GLE_IF(FALSE, seekStatus, "Seek to end offset in existing log file");
	}
}

AnsiFileLogSink::~AnsiFileLogSink()
{
	CloseHandle(m_logfile);
}

void AnsiFileLogSink::log(const std::wstring &message)
{
	auto ansi = common::string::ToAnsi(message);

	ansi.append("\xd\xa");

	DWORD bytesWritten;

	WriteFile(m_logfile, ansi.c_str(), ansi.size(), &bytesWritten, nullptr);

	if (m_flush)
	{
		FlushFileBuffers(m_logfile);
	}
}

void Logger::log(const std::wstring &message)
{
	m_logsink->log(Compose(message, Timestamp(), ordinal()));
}

void Logger::log(const std::wstring &message, const std::vector<std::wstring> &details)
{
	const auto timestamp = this->Timestamp();
	const auto ordinal = this->ordinal();

	m_logsink->log(Compose(message, timestamp, ordinal));

	//
	// Write details with indentation.
	//
	for (const auto detail : details)
	{
		m_logsink->log(Compose(detail, timestamp, ordinal, 4));
	}
}

// static
std::wstring Logger::Timestamp()
{
	SYSTEMTIME time;

	GetLocalTime(&time);

	std::wstringstream ss;

	ss << L'['
		<< std::right << std::setw(2) << std::setfill(L'0') << time.wHour
		<< L':'
		<< std::right << std::setw(2) << std::setfill(L'0') << time.wMinute
		<< L':'
		<< std::right << std::setw(2) << std::setfill(L'0') << time.wSecond
		<< L']';

	return ss.str();
}

std::wstring Logger::ordinal()
{
	std::wstringstream ss;

	ss << std::right << std::setw(4) << std::setfill(L' ') << m_ordinal++;

	return ss.str();
}

//static
std::wstring Logger::Compose(const std::wstring &message, const std::wstring &timestamp,
	const std::wstring &ordinal, size_t indentation)
{
	std::wstringstream ss;

	ss << timestamp << L' '
		<< ordinal << L' '
		<< std::wstring(indentation, L' ')
		<< message;

	return ss.str();
}
