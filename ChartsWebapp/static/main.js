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
                format: '{value:%a %l%P}',
            },
        },
        yAxis: [{ // Primary yAxis
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
        }, { // Secondary yAxis
            title: {
                text: 'Number Of Donations',
                style: {
                    color: Highcharts.getOptions().colors[1]
                }
            },
            labels: {
                format: '{value:,.0f}',
                style: {
                    color: Highcharts.getOptions().colors[1]
                }
            },
            opposite: true
        }],
        tooltip: {
            formatter: function () {
            	if(this.series.name == "Donation Total") {
	                return '<b>' + this.series.name + '</b><br/>' +
	                    Highcharts.dateFormat('%A, %b %e %l:%M%P', this.x) + '<br/>$' +
	                    Highcharts.numberFormat(this.y, 0);
                } else {
                	return '<b>' + this.series.name + '</b><br/>' +
	                    Highcharts.dateFormat('%A, %b %e %l:%M%P', this.x) + '<br/>' +
	                    Highcharts.numberFormat(this.y, 0,"","");
                }
            }
        },
        legend: {
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
            name: 'Number Of Donations',
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