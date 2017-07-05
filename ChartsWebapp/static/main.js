var data_url = "/donation_data";
var data_update_url = "/donation_data/update";

$(function() {
	$.get(data_url).done(function(fullData) {
        initWithData(fullData);

        //now that we have our charts, start refreshing the data every 5 minutes
        var mostRecentUpdate = new Date();

        var performUpdate = function() {
            $.get(data_update_url, {'since':mostRecentUpdate.toISOString()}).done(function(partialData) {

                //only redraw the chart if there's something to redraw
                if(partialData.length > 0) {
                    fullData = fullData.concat(partialData);
                    initWithData(fullData);
                }

                mostRecentUpdate = new Date();

                //do another update in 5 minutes
                setTimeout(performUpdate, getRefreshTimeout());
            });
        }
        setTimeout(performUpdate, getRefreshTimeout());
    });

    //auto refresh the whole page after 2 hours
    setTimeout(function() {
        window.location.reload(true);
    }, 2 * 60 * 60 * 1000);
});

var getRefreshTimeout = function() {
    //5 minutes, plus a random value between 0 and 30 seconds
    return 5 * 60 * 1000 + 30 *1000 * Math.random();
}


var initWithData = function(donationData) {
	var current_totals = donationData.map(function(item) {
        return item.total;
    });
    var old_totals = donationData.map(function(item) {
        return item.total_2016;
    });
	var timestamps = donationData.map(function(item) {
		return Date.parse(item.timestamp);
	});

	//create a series for dollars per minute
	var hourlyTimeStamps
	var dollarsPerHour = [];
	var dollarsPerDonation = [];
	for (var i = 0; i < current_totals.length - 1; i++) {
		var deltaTotal = current_totals[i + 1] - current_totals[i];
		var hoursBetweenSamples = (timestamps[i + 1] - timestamps[i]) / (1000 * 60 * 60);
		dollarsPerHour.push(deltaTotal / (hoursBetweenSamples > 0 ? hoursBetweenSamples : 1));
	}
	dollarsPerHour = ironOutTopPercent(dollarsPerHour, 0.025);

	//data series for dollars per donation
	var dollarsPerDonation = [];
	for (var i = 0; i < current_totals.length - 1; i++) {
		var deltaTotal = current_totals[i + 1] - current_totals[i];
		var deltaCount = donationData[i + 1].count - donationData[i].count;
		dollarsPerDonation.push(deltaTotal / (deltaCount > 0 ? deltaCount : 1));
	}
	dollarsPerDonation = ironOutTopPercent(dollarsPerDonation, 0.025);

	//create our various charts
	var totals_chart = createTotalsChart(timestamps, current_totals, old_totals);

	//remove the first timestamp since we'll no longer be using it
	timestamps.splice(0,1);
	var rate_chart = createRateChart(timestamps, dollarsPerHour);
	var per_donation_chart = createPerDonationChart(timestamps, dollarsPerDonation);
}

var createTotalsChart = function(timestamps, totals, old_totals) {
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
            text: 'gdq-charts.xyz - Unofficial SGDQ 2017 Donation Chart',
        },
        subtitle: {
            text: "No refresh necessary; Data auto-updates every 15 minutes",
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
            min: 0,
        }, {
            linkedTo: 0,
            opposite: true,
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
        }],
        tooltip: {
            formatter: function () {
                return '<b>' + this.series.name + '</b><br/>' +
                    Highcharts.dateFormat('%A, %b %e %l:%M%P', this.x) + '<br/>$' +
                    Highcharts.numberFormat(this.y, 0);
            }
        },
        legend: {
            enabled: true,
            layout: 'vertical',
            align: 'right',
            verticalAlign: 'top',
            x: -200,
            floating: true,
        },
        series: [{
            name: 'Donation Total (Current)',
            data: _.zip(timestamps, totals),
            color: Highcharts.getOptions().colors[0]
        },{
            name: 'Donation Total (2016)',
            data: _.zip(timestamps, old_totals),
            color: Highcharts.getOptions().colors[3]
        },

        ]
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
            text: '$ Per Hour'
        },
        subtitle: {
            text: "Average, top 2.5% donations removed",
        },
        xAxis: {
            type: 'datetime',
            labels: {
                format: '{value:%a %l%P}',
            },
        },
        yAxis: [{ // Primary yAxis
            title: {
                text: '$ Per Hour',
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
            min: 0,
        }, {
            linkedTo:0,
            opposite:true,
            title: {
                text: '$ Per Hour',
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
            min: 0,
        }],
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
            name: '$ Per Hour [Average]',
            data: _.zip(timestamps, rate_data),
            color: Highcharts.getOptions().colors[1]
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
        subtitle: {
            text: "Average, top 2.5% donations removed",
        },
        xAxis: {
            type: 'datetime',
            labels: {
                format: '{value:%a %l%P}',
            },
        },
        yAxis: [{ // Primary yAxis
            title: {
                text: '$ Per Donation [Average]',
                style: {
                    color: Highcharts.getOptions().colors[2]
                }
            },
            labels: {
                format: '${value:,.2f}',
                style: {
                    color: Highcharts.getOptions().colors[2]
                }
            },
            min:0,
        }, {
            linkedTo: 0,
            opposite: true,
            title: {
                text: '$ Per Donation [Average]',
                style: {
                    color: Highcharts.getOptions().colors[2]
                }
            },
            labels: {
                format: '${value:,.2f}',
                style: {
                    color: Highcharts.getOptions().colors[2]
                }
            },
            min:0,
        }],
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
            name: '$ Per Donation [Average]',
            data: _.zip(timestamps, per_donation_data),
            color: Highcharts.getOptions().colors[2]
        }]
    });
};

//replaces the largest n% of the numbers in the provided list with an average of the numbers to the left and right
//the goal is to provide a way to eliminate outliers
var ironOutTopPercent = function(listOfNumbers, topPercent) {
    var size = listOfNumbers.length;
	var topValues = topN(listOfNumbers, Math.floor(topPercent * size));
	topValues = arrayToObjectKeys(topValues);

	for(var i = 1; i < size - 1; i++) {
		if(listOfNumbers[i] in topValues) {
			listOfNumbers[i] = (listOfNumbers[i - 1] + listOfNumbers[i + 1])/2;
		}
	}

    if(listOfNumbers[size - 1] in topValues) {
        listOfNumbers[size - 1] = (listOfNumbers[size - 2] + listOfNumbers[size - 3])/2;
    }

    var averageWeights = [0.1, 0.2, 0.4, 0.2, 0.1];

    return listOfNumbers.map(function(element, i) {
        return averagedSample(listOfNumbers, averageWeights, i);
    });
}

var averagedSample = function(inputArray, weightArray, index) {
    var average = 0;

    for(var i = 0; i < weightArray.length; i++) {
        var sampleIndex = index - Math.floor(weightArray.length / 2) + i;
        var sample;

        if(sampleIndex < 0) {
            sample = inputArray[0];
        } else if(sampleIndex >= inputArray.length) {
            sample = inputArray[inputArray.length - 1];
        } else {
            sample = inputArray[sampleIndex];
        }

        average += weightArray[i] * sample;
    }

    return average;
}

//given a list of numbers and n, return the n largest values of listOfNumbers, in sorted order
var topN = function(listOfNumbers, n) {
	var values = listOfNumbers.slice(0, n);
	values.sort();

	for(var i = n; i < listOfNumbers.length; i++) {
		var insertIndex = _.sortedIndex(values, listOfNumbers[i]);
		if(insertIndex > 0) {
			values.splice(insertIndex, 0, listOfNumbers[i]);
			values.splice(0, 1);
		}
	}

	return values;
}

var arrayToObjectKeys = function(arr) {
	return arr.reduce(function(acc, cur, i) {
		acc[cur] = true;
		return acc;
	}, {});
}