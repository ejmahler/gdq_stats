var data_url = "/donation_data";
var data_update_url = "/donation_data_update";

$(function() {
	$.get(data_url)
	.done(initWithData)
	.fail(function() {
		$("#data-container").text("Failed to load data");
	});
});

var initWithData = function(donationData) {
	var totals = donationData.map(function(item) {
		return item.total;
	});
	var timestamps = donationData.map(function(item) {
		return Date.parse(item.timestamp);
	});

	var dollarsPerMinute = [null];
	for (var i = 0; i < totals.length - 1; i++) {
		var deltaTotal = totals[i + 1] - totals[i];
		var minutesBetweenSamples = (timestamps[i + 1] - timestamps[i]) / (1000 * 60);
		dollarsPerMinute.push(deltaTotal / minutesBetweenSamples);
	}

	var dollarsPerDonation = [null];
	for (var i = 0; i < totals.length - 1; i++) {
		var deltaTotal = totals[i + 1] - totals[i];
		var deltaCount = donationData[i + 1].count - donationData[i].count;
		dollarsPerDonation.push(deltaTotal / deltaCount);
	}

	var totals_chart = createTotalsChart(timestamps, totals);
	var rate_chart = createRateChart(timestamps, dollarsPerMinute);
	var per_donation_chart = createPerDonationChart(timestamps, dollarsPerDonation);
}

var createTotalsChart = function(timestamps, totals) {
	Highcharts.setOptions({
        global: {
            useUTC: false
        }
    });

    return Highcharts.chart('total-container', {
        chart: {
            type: 'spline',
            animation: Highcharts.svg,
        },
        title: {
            text: 'Unofficial AGDQ 2017 Donation Chart'
        },
        xAxis: {
            type: 'datetime',
            labels: {
                format: '{value:%a %l%P}',
            },
        },
        yAxis: { // Primary yAxis
            title: {
                text: 'Donation Total',
                style: {
                    color: Highcharts.getOptions().colors[0]
                }
            },
            labels: {
                format: '${value:,.0f}',
                style: {
                    color: Highcharts.getOptions().colors[0]
                }
            },
        },
        tooltip: {
            formatter: function () {
                return '<b>' + this.series.name + '</b><br/>' +
                    Highcharts.dateFormat('%A, %b %e %l:%M%P', this.x) + '<br/>$' +
                    Highcharts.numberFormat(this.y, 0);
            }
        },
        legend: {
            enabled: false
        },
        series: [{
            name: 'Donation Total',
            data: _.zip(timestamps, totals)
        }]
    });
};

var createRateChart = function(timestamps, rate_data) {
	Highcharts.setOptions({
        global: {
            useUTC: false
        }
    });

    return Highcharts.chart('rate-container', {
        chart: {
            type: 'spline',
            animation: Highcharts.svg,
        },
        title: {
            text: '$ Per Minute'
        },
        xAxis: {
            type: 'datetime',
            labels: {
                format: '{value:%a %l%P}',
            },
        },
        yAxis: { // Primary yAxis
            title: {
                text: '$ Per Minute',
                style: {
                    color: Highcharts.getOptions().colors[0]
                }
            },
            labels: {
                format: '${value:,.0f}',
                style: {
                    color: Highcharts.getOptions().colors[0]
                }
            },
        },
        tooltip: {
            formatter: function () {
                return '<b>' + this.series.name + '</b><br/>' +
                    Highcharts.dateFormat('%A, %b %e %l:%M%P', this.x) + '<br/>$' +
                    Highcharts.numberFormat(this.y, 0);
            }
        },
        legend: {
            enabled: false
        },
        series: [{
            name: '$ Per Minute',
            data: _.zip(timestamps, rate_data)
        }]
    });
};

var createPerDonationChart = function(timestamps, per_donation_data) {
	Highcharts.setOptions({
        global: {
            useUTC: false
        }
    });

    return Highcharts.chart('per-donation-container', {
        chart: {
            type: 'spline',
            animation: Highcharts.svg,
        },
        title: {
            text: '$ Per Donation'
        },
        xAxis: {
            type: 'datetime',
            labels: {
                format: '{value:%a %l%P}',
            },
        },
        yAxis: { // Primary yAxis
            title: {
                text: '$ Per Donation',
                style: {
                    color: Highcharts.getOptions().colors[0]
                }
            },
            labels: {
                format: '${value:,.2f}',
                style: {
                    color: Highcharts.getOptions().colors[0]
                }
            },
        },
        tooltip: {
            formatter: function () {
                return '<b>' + this.series.name + '</b><br/>' +
                    Highcharts.dateFormat('%A, %b %e %l:%M%P', this.x) + '<br/>$' +
                    Highcharts.numberFormat(this.y, 0);
            }
        },
        legend: {
            enabled: false
        },
        series: [{
            name: '$ Per Donation',
            data: _.zip(timestamps, per_donation_data)
        }]
    });
};