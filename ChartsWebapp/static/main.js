var data_url = "/donation_data";
var data_update_url = "/donation_data_update";

var createChart = function(donation_data) {
	Highcharts.setOptions({
        global: {
            useUTC: false
        }
    });

    Highcharts.chart('chart-container', {
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
                format: '{value:%b-%d %H:%M}',
                style: {
                    color: Highcharts.getOptions().colors[1]
                }
            },
        },
        yAxis: [{ // Primary yAxis
            title: {
                text: 'Donation Total',
                style: {
                    color: Highcharts.getOptions().colors[1]
                }
            },
            labels: {
                format: '${value:,.0f}',
                style: {
                    color: Highcharts.getOptions().colors[1]
                }
            },
        }, { // Secondary yAxis
            title: {
                text: 'Donation Count',
                style: {
                    color: Highcharts.getOptions().colors[0]
                }
            },
            labels: {
                format: '{value:,.0f}',
                style: {
                    color: Highcharts.getOptions().colors[0]
                }
            },
            opposite: true
        }],
        legend: {
            enabled: false
        },
        exporting: {
            enabled: false
        },
        series: [{
            name: 'Donation Total',
            yAxis: 0,
            data: (function () {
            	var series_data = [];
            	donation_data.forEach(function(value) {
            		series_data.push([Date.parse(value.timestamp), value.total]);
            	});
                return series_data;
            }())
        },{
            name: 'Donation Count',
            yAxis: 1,
            data: (function () {
            	var series_data = [];
            	donation_data.forEach(function(value) {
            		series_data.push([Date.parse(value.timestamp), value.count]);
            	});
                return series_data;
            }())
        }]
    });
};

$(function() {
	$.get(data_url)
	.done(createChart)
	.fail(function() {
		$("#data-container").text("Failed to load data");
	});
});